'use client';

import React, { Component, ErrorInfo, ReactNode } from 'react';
import { AlertTriangle, RefreshCw, Home } from 'lucide-react';

interface Props {
    children: ReactNode;
    fallback?: ReactNode;
    resetKeys?: any[];
    onReset?: () => void;
    name?: string;
}

interface State {
    hasError: boolean;
    error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
    public state: State = {
        hasError: false,
        error: null,
    };

    public static getDerivedStateFromError(error: Error): State {
        return { hasError: true, error };
    }

    public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
        console.error(`ErrorBoundary caught an error in [${this.props.name || 'Unknown'}]:`, error, errorInfo);
    }

    public componentDidUpdate(prevProps: Props) {
        if (this.state.hasError && this.props.resetKeys) {
            if (this.hasChanged(prevProps.resetKeys, this.props.resetKeys)) {
                this.reset();
            }
        }
    }

    private hasChanged(prev?: any[], next?: any[]) {
        if (!prev || !next) return false;
        if (prev.length !== next.length) return true;
        for (let i = 0; i < prev.length; i++) {
            if (prev[i] !== next[i]) return true;
        }
        return false;
    }

    private reset = () => {
        this.setState({ hasError: false, error: null });
        this.props.onReset?.();
    };

    public render() {
        if (this.state.hasError) {
            if (this.props.fallback) {
                return this.props.fallback;
            }

            return (
                <div className="min-h-[400px] flex items-center justify-center p-6">
                    <div className="max-w-md w-full bg-white rounded-2xl shadow-sm border border-red-100 p-8 text-center">
                        <div className="w-16 h-16 bg-red-50 rounded-full flex items-center justify-center mx-auto mb-6">
                            <AlertTriangle className="w-8 h-8 text-red-600" />
                        </div>

                        <h2 className="text-2xl font-bold text-gray-900 mb-2">Something went wrong</h2>
                        <p className="text-gray-600 mb-8">
                            We encountered an unexpected error while interaction with the Stellar network.
                            {this.state.error && (
                                <span className="block mt-2 text-xs font-mono text-red-500 bg-red-50 p-2 rounded truncate">
                                    {this.state.error.message}
                                </span>
                            )}
                        </p>

                        <div className="flex flex-col gap-3">
                            <button
                                onClick={this.reset}
                                className="flex items-center justify-center gap-2 w-full py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors font-medium"
                            >
                                <RefreshCw className="w-4 h-4" />
                                Try Again
                            </button>

                            <a
                                href="/"
                                className="flex items-center justify-center gap-2 w-full py-3 bg-white border border-gray-200 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors font-medium"
                            >
                                <Home className="w-4 h-4" />
                                Return Home
                            </a>
                        </div>
                    </div>
                </div>
            );
        }

        return this.props.children;
    }
}
