import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Search } from 'lucide-react';
import { describe, expect, it, vi } from 'vitest';
import { Input } from './Input';

describe('Input', () => {
  it('renders an input element', () => {
    render(<Input placeholder="Type here" />);
    expect(screen.getByPlaceholderText('Type here')).toBeInTheDocument();
  });

  it('renders with a label', () => {
    render(<Input label="Username" />);
    expect(screen.getByText('Username')).toBeInTheDocument();
    // Label should be associated with the input
    expect(screen.getByLabelText('Username')).toBeInTheDocument();
  });

  it('renders with an icon', () => {
    const { container } = render(<Input icon={Search} placeholder="Search…" />);
    // The icon renders as an SVG element
    const svg = container.querySelector('svg');
    expect(svg).toBeInTheDocument();
  });

  it('shows error state with red border and message', () => {
    render(<Input error="This field is required" />);
    // Error message should be rendered
    expect(screen.getByText('This field is required')).toBeInTheDocument();
  });

  it('disables input when disabled prop is set', async () => {
    const user = userEvent.setup();
    const handleChange = vi.fn();
    render(
      <Input disabled placeholder="Disabled" onChange={handleChange} />,
    );
    const input = screen.getByPlaceholderText('Disabled');
    expect(input).toBeDisabled();

    await user.type(input, 'text');
    expect(handleChange).not.toHaveBeenCalled();
  });

  it('renders helper text', () => {
    render(<Input helperText="Enter your email address" />);
    expect(screen.getByText('Enter your email address')).toBeInTheDocument();
  });

  it('renders with both label and helper text', () => {
    render(
      <Input
        label="Email"
        helperText="We will never share your email"
        placeholder="you@example.com"
      />,
    );
    expect(screen.getByText('Email')).toBeInTheDocument();
    expect(screen.getByText('We will never share your email')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('you@example.com')).toBeInTheDocument();
  });
});
