import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { SearchInput } from './SearchInput';

describe('SearchInput', () => {
  it('renders with default placeholder', () => {
    render(<SearchInput value="" onChange={() => {}} />);
    expect(screen.getByPlaceholderText('Search...')).toBeInTheDocument();
  });

  it('renders with a custom placeholder', () => {
    render(
      <SearchInput placeholder="Find memories..." value="" onChange={() => {}} />,
    );
    expect(
      screen.getByPlaceholderText('Find memories...'),
    ).toBeInTheDocument();
  });

  it('shows clear button when value is non-empty and calls onClear', async () => {
    const user = userEvent.setup();
    const handleClear = vi.fn();

    render(
      <SearchInput value="test" onChange={() => {}} onClear={handleClear} />,
    );

    const clearButton = screen.getByLabelText('Clear search');
    expect(clearButton).toBeInTheDocument();

    await user.click(clearButton);
    expect(handleClear).toHaveBeenCalledTimes(1);
  });

  it('does not show clear button when value is empty', () => {
    render(<SearchInput value="" onChange={() => {}} />);
    expect(screen.queryByLabelText('Clear search')).not.toBeInTheDocument();
  });

  it('shows keyboard shortcut hint when value is empty and shortcut is provided', () => {
    render(
      <SearchInput value="" onChange={() => {}} shortcut="⌘K" />,
    );
    expect(screen.getByText('⌘K')).toBeInTheDocument();
  });

  it('hides shortcut hint when value is non-empty', () => {
    render(
      <SearchInput value="query" onChange={() => {}} shortcut="⌘K" />,
    );
    expect(screen.queryByText('⌘K')).not.toBeInTheDocument();
  });

  it('calls onChange when the user types', async () => {
    const user = userEvent.setup();
    const handleChange = vi.fn();

    render(<SearchInput value="" onChange={handleChange} />);
    const input = screen.getByPlaceholderText('Search...');
    await user.type(input, 'a');

    expect(handleChange).toHaveBeenCalled();
  });
});
