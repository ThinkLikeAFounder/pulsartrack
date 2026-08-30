import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { TargetingForm } from './TargetingForm';

describe('TargetingForm', () => {
    it('prevents duplicate submissions while a save is pending', async () => {
        let resolveSave: (() => void) | undefined;
        const onSave = vi.fn(
            () =>
                new Promise<void>((resolve) => {
                    resolveSave = resolve;
                }),
        );

        render(<TargetingForm onSave={onSave} />);

        fireEvent.change(screen.getByLabelText(/Min Age/i), {
            target: { value: '21' },
        });

        const submitButton = await screen.findByRole('button', {
            name: /Save Targeting Settings/i,
        });

        fireEvent.click(submitButton);
        fireEvent.click(submitButton);

        await waitFor(() => {
            expect(onSave).toHaveBeenCalledTimes(1);
        });

        expect(screen.getByRole('button', { name: /Saving\.\.\./i })).toBeDisabled();

        resolveSave?.();

        await waitFor(() => {
            expect(screen.getByText(/Targeting settings saved!/i)).toBeInTheDocument();
        });
    });
});
