import { render, screen, act, fireEvent } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ToastProvider, useToast } from './ToastContext';

function ToastHarness() {
  const { success } = useToast();

  return (
    <button onClick={() => success('Saved', 'Changes stored')}>
      Trigger toast
    </button>
  );
}

describe('ToastProvider', () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it('auto-dismisses a toast after 5 seconds', () => {
    vi.useFakeTimers();

    render(
      <ToastProvider>
        <ToastHarness />
      </ToastProvider>
    );

    fireEvent.click(screen.getByText('Trigger toast'));
    expect(screen.getByText('Saved')).toBeInTheDocument();

    act(() => {
      vi.advanceTimersByTime(5000);
    });

    expect(screen.queryByText('Saved')).not.toBeInTheDocument();
  });

  it('clears the auto-dismiss timer when a toast is manually dismissed', () => {
    vi.useFakeTimers();
    const clearTimeoutSpy = vi.spyOn(globalThis, 'clearTimeout');

    render(
      <ToastProvider>
        <ToastHarness />
      </ToastProvider>
    );

    fireEvent.click(screen.getByText('Trigger toast'));
    fireEvent.click(screen.getByRole('button', { name: '✕' }));

    expect(screen.queryByText('Saved')).not.toBeInTheDocument();
    expect(clearTimeoutSpy).toHaveBeenCalledTimes(1);

    act(() => {
      vi.advanceTimersByTime(5000);
    });

    expect(screen.queryByText('Saved')).not.toBeInTheDocument();
  });

  it('cleans up outstanding timers on unmount', () => {
    vi.useFakeTimers();
    const clearTimeoutSpy = vi.spyOn(globalThis, 'clearTimeout');

    const { unmount } = render(
      <ToastProvider>
        <ToastHarness />
      </ToastProvider>
    );

    fireEvent.click(screen.getByText('Trigger toast'));
    expect(screen.getByText('Saved')).toBeInTheDocument();

    unmount();

    expect(clearTimeoutSpy).toHaveBeenCalledTimes(1);
    expect(() => {
      act(() => {
        vi.runAllTimers();
      });
    }).not.toThrow();
  });
});
