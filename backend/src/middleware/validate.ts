import { Request, Response, NextFunction } from 'express';
import { StrKey } from '@stellar/stellar-sdk';
import { URL } from 'url';

interface FieldRule {
  type: 'string' | 'number' | 'stellar_address';
  format?: 'url';
  required?: boolean;
  min?: number;
  max?: number;
  minLength?: number;
  maxLength?: number;
  integer?: boolean;
}

interface ValidationSchema {
  params?: Record<string, FieldRule>;
  query?: Record<string, FieldRule>;
  body?: Record<string, FieldRule>;
}

interface ValidationError {
  field: string;
  message: string;
}

function validateField(value: unknown, field: string, rule: FieldRule): ValidationError | null {
  if (value === undefined || value === null || value === '') {
    if (rule.required) {
      return { field, message: `${field} is required` };
    }
    return null;
  }

  if (rule.type === 'stellar_address') {
    if (!StrKey.isValidEd25519PublicKey(String(value))) {
      return { field, message: `${field} must be a valid Stellar public key` };
    }
  }

  if (rule.type === 'number') {
    const num = Number(value);
    if (isNaN(num)) {
      return { field, message: `${field} must be a number` };
    }
    if (rule.integer && !Number.isInteger(num)) {
      return { field, message: `${field} must be an integer` };
    }
    if (rule.min !== undefined && num < rule.min) {
      return { field, message: `${field} must be at least ${rule.min}` };
    }
    if (rule.max !== undefined && num > rule.max) {
      return { field, message: `${field} must be at most ${rule.max}` };
    }
  }

  if (rule.type === 'string') {
    const str = String(value);
    if (rule.minLength !== undefined && str.length < rule.minLength) {
      return { field, message: `${field} must be at least ${rule.minLength} characters` };
    }
    if (rule.maxLength !== undefined && str.length > rule.maxLength) {
      return { field, message: `${field} must be at most ${rule.maxLength} characters` };
    }

    if (rule.format === 'url' && str) {
      try {
        const url = new URL(str);
        if (!['http:', 'https:'].includes(url.protocol)) {
          return { field, message: `${field} must be an HTTP(S) URL` };
        }
      } catch {
        return { field, message: `${field} must be a valid URL` };
      }
    }
  }

  return null;
}

export function validate(schema: ValidationSchema) {
  return (req: Request, res: Response, next: NextFunction): void => {
    const errors: ValidationError[] = [];

    for (const [source, rules] of Object.entries(schema)) {
      const data =
        (req as unknown as Record<string, Record<string, unknown> | undefined>)[source] ?? {};
      for (const [field, rule] of Object.entries(rules as Record<string, FieldRule>)) {
        const error = validateField(data[field], field, rule);
        if (error) errors.push(error);
      }
    }

    if (errors.length > 0) {
      res.status(400).json({ error: 'Validation failed', details: errors });
      return;
    }

    next();
  };
}
