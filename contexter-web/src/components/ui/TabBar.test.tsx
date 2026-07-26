import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { TabBar } from './TabBar';

const tabs = [
  { id: 'overview', label: 'Overview' },
  { id: 'sessions', label: 'Sessions' },
  { id: 'efficiency', label: 'Efficiency' },
];

describe('TabBar', () => {
  it('renders all tab labels', () => {
    render(<TabBar tabs={tabs} activeTab="overview" onChange={() => {}} />);
    for (const tab of tabs) {
      expect(screen.getByText(tab.label)).toBeInTheDocument();
    }
  });

  it('applies active state to the selected tab', () => {
    render(<TabBar tabs={tabs} activeTab="sessions" onChange={() => {}} />);
    const activeButton = screen.getByText('Sessions');
    expect(activeButton).toHaveAttribute('aria-selected', 'true');
  });

  it('sets aria-selected false for inactive tabs', () => {
    render(<TabBar tabs={tabs} activeTab="overview" onChange={() => {}} />);
    const sessionsButton = screen.getByText('Sessions');
    expect(sessionsButton).toHaveAttribute('aria-selected', 'false');
  });

  it('calls onChange with tab id when clicked', async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    render(<TabBar tabs={tabs} activeTab="overview" onChange={onChange} />);

    await user.click(screen.getByText('Sessions'));
    expect(onChange).toHaveBeenCalledWith('sessions');
  });

  it('renders as a tablist role', () => {
    render(<TabBar tabs={tabs} activeTab="overview" onChange={() => {}} />);
    expect(screen.getByRole('tablist')).toBeInTheDocument();
  });

  it('renders each tab with tab role', () => {
    render(<TabBar tabs={tabs} activeTab="overview" onChange={() => {}} />);
    const tabButtons = screen.getAllByRole('tab');
    expect(tabButtons).toHaveLength(3);
  });

  it('applies custom className', () => {
    const { container } = render(
      <TabBar tabs={tabs} activeTab="overview" onChange={() => {}} className="custom-tabs" />,
    );
    const tablist = container.firstChild as HTMLElement;
    expect(tablist.className).toContain('custom-tabs');
  });
});
