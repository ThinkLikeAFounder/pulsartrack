import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { TargetingForm } from './TargetingForm';

const SUBMIT_NAME = /Save Targeting Settings/i;

async function submitForm() {
    const submitButton = await screen.findByRole('button', { name: SUBMIT_NAME });
    await waitFor(() => expect(submitButton).toBeEnabled());
    fireEvent.click(submitButton);
}

describe('TargetingForm', () => {
    describe.each([
        { field: 'regions', addLabel: 'Add Target Regions', value: 'US' },
        { field: 'interests', addLabel: 'Add Interest Segments', value: 'technology' },
        { field: 'devices', addLabel: 'Add Device Types', value: 'tablet' },
        { field: 'languages', addLabel: 'Add Languages', value: 'fr' },
        { field: 'excludedSegments', addLabel: 'Add Excluded Segments', value: 'gaming' },
    ])('$field field array', ({ addLabel, value }) => {
        it(`adds "${value}" via "${addLabel}" and removes it again`, () => {
            render(<TargetingForm />);
            const removeName = new RegExp(`^Remove ${value}$`);

            expect(screen.queryByRole('button', { name: removeName })).not.toBeInTheDocument();

            fireEvent.change(screen.getByLabelText(addLabel), { target: { value } });
            expect(screen.getByRole('button', { name: removeName })).toBeInTheDocument();

            fireEvent.click(screen.getByRole('button', { name: removeName }));
            expect(screen.queryByRole('button', { name: removeName })).not.toBeInTheDocument();
        });

        it(`does not offer "${value}" again once it has been added`, () => {
            render(<TargetingForm />);
            const select = screen.getByLabelText(addLabel) as HTMLSelectElement;
            const optionValues = () => Array.from(select.options).map((o) => o.value);

            expect(optionValues()).toContain(value);
            fireEvent.change(select, { target: { value } });
            expect(optionValues()).not.toContain(value);
        });
    });

    it('does not submit the form when a segment is removed', async () => {
        const onSave = vi.fn().mockResolvedValue(undefined);
        render(<TargetingForm onSave={onSave} />);

        // "desktop" is a default device
        fireEvent.click(screen.getByRole('button', { name: /^Remove desktop$/ }));

        await new Promise((resolve) => setTimeout(resolve, 0));
        expect(onSave).not.toHaveBeenCalled();
    });

    it('keeps an interest out of the excluded-segment options', () => {
        render(<TargetingForm />);
        const excluded = screen.getByLabelText('Add Excluded Segments') as HTMLSelectElement;

        expect(Array.from(excluded.options).map((o) => o.value)).toContain('technology');
        fireEvent.change(screen.getByLabelText('Add Interest Segments'), {
            target: { value: 'technology' },
        });
        expect(Array.from(excluded.options).map((o) => o.value)).not.toContain('technology');
    });

    it('seeds the segment lists from the initial config', () => {
        render(
            <TargetingForm
                initial={{ regions: ['EU'], interests: ['finance'], devices: ['tablet'], languages: ['de'] }}
            />,
        );

        for (const name of ['EU', 'finance', 'tablet', 'de']) {
            expect(screen.getByRole('button', { name: `Remove ${name}` })).toBeInTheDocument();
        }
        // initial devices/languages replace the defaults
        expect(screen.queryByRole('button', { name: 'Remove desktop' })).not.toBeInTheDocument();
        expect(screen.queryByRole('button', { name: 'Remove en' })).not.toBeInTheDocument();
    });

    it('passes correctly-shaped targeting parameters to onSave', async () => {
        const onSave = vi.fn().mockResolvedValue(undefined);
        render(<TargetingForm onSave={onSave} />);

        fireEvent.change(screen.getByLabelText('Add Target Regions'), { target: { value: 'US' } });
        fireEvent.change(screen.getByLabelText('Add Target Regions'), { target: { value: 'EU' } });
        fireEvent.change(screen.getByLabelText('Add Interest Segments'), { target: { value: 'technology' } });
        fireEvent.change(screen.getByLabelText('Add Excluded Segments'), { target: { value: 'gaming' } });
        fireEvent.change(screen.getByLabelText('Add Device Types'), { target: { value: 'tablet' } });
        fireEvent.change(screen.getByLabelText('Add Languages'), { target: { value: 'fr' } });
        fireEvent.click(screen.getByRole('button', { name: /^Remove mobile$/ }));

        fireEvent.change(screen.getByLabelText(/Min Age/i), { target: { value: '25' } });
        fireEvent.change(screen.getByLabelText(/Max Age/i), { target: { value: '40' } });
        fireEvent.change(screen.getByLabelText(/Min Publisher Reputation/i), { target: { value: '100' } });
        fireEvent.change(screen.getByLabelText(/Max CPM/i), { target: { value: '0.5' } });
        fireEvent.click(screen.getByRole('checkbox', { name: /Require KYC/i }));

        await submitForm();

        await waitFor(() => expect(onSave).toHaveBeenCalledTimes(1));
        expect(onSave.mock.calls[0][0]).toEqual({
            regions: ['US', 'EU'],
            interests: ['technology'],
            excludedSegments: ['gaming'],
            devices: ['desktop', 'tablet'],
            languages: ['en', 'fr'],
            minAge: 25,
            maxAge: 40,
            minReputation: 100,
            requireKyc: true,
            excludeFraud: true,
            maxCpmXlm: '0.5',
        });
    });

    it('rejects a max age lower than the min age and blocks submission', async () => {
        const onSave = vi.fn().mockResolvedValue(undefined);
        render(<TargetingForm onSave={onSave} />);

        fireEvent.change(screen.getByLabelText(/Min Age/i), { target: { value: '50' } });
        fireEvent.change(screen.getByLabelText(/Max Age/i), { target: { value: '30' } });
        fireEvent.blur(screen.getByLabelText(/Max Age/i));

        expect(
            await screen.findByText(/Max age must be greater than or equal to min age/i),
        ).toBeInTheDocument();
        expect(screen.getByRole('button', { name: SUBMIT_NAME })).toBeDisabled();
        expect(onSave).not.toHaveBeenCalled();
    });

    it('accepts equal min and max ages', async () => {
        const onSave = vi.fn().mockResolvedValue(undefined);
        render(<TargetingForm onSave={onSave} />);

        fireEvent.change(screen.getByLabelText(/Min Age/i), { target: { value: '30' } });
        fireEvent.change(screen.getByLabelText(/Max Age/i), { target: { value: '30' } });
        fireEvent.blur(screen.getByLabelText(/Max Age/i));

        await submitForm();

        await waitFor(() => expect(onSave).toHaveBeenCalledTimes(1));
        expect(onSave.mock.calls[0][0]).toMatchObject({ minAge: 30, maxAge: 30 });
        expect(
            screen.queryByText(/Max age must be greater than or equal to min age/i),
        ).not.toBeInTheDocument();
    });

    it('rejects an age below the minimum of 13', async () => {
        render(<TargetingForm />);

        fireEvent.change(screen.getByLabelText(/Min Age/i), { target: { value: '10' } });
        fireEvent.blur(screen.getByLabelText(/Min Age/i));

        expect(await screen.findByText(/Min age must be at least 13/i)).toBeInTheDocument();
        expect(screen.getByRole('button', { name: SUBMIT_NAME })).toBeDisabled();
    });

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
