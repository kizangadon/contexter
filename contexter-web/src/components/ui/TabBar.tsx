import { type ReactNode } from 'react';

export interface Tab {
  /** Unique identifier for the tab */
  id: string;
  /** Display label */
  label: string;
  /** Optional icon rendered before label */
  icon?: ReactNode;
}

export interface TabBarProps {
  /** Array of tab definitions */
  tabs: Tab[];
  /** Currently active tab ID */
  activeTab: string;
  /** Called when a tab is selected */
  onChange: (tabId: string) => void;
  /** Additional CSS class names */
  className?: string;
}

const baseStyles =
  'inline-flex items-center gap-2 rounded-md px-4 py-2 text-sm font-medium transition-colors duration-150 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent';

const activeStyles = 'bg-accent text-text-inverse';
const inactiveStyles =
  'text-text-secondary hover:bg-bg-hover hover:text-text-primary';

export function TabBar({
  tabs,
  activeTab,
  onChange,
  className = '',
}: TabBarProps) {
  return (
    <div
      role="tablist"
      className={`flex flex-wrap items-center gap-1 rounded-lg bg-bg-tertiary p-1 ${className}`}
    >
      {tabs.map((tab) => (
        <button
          key={tab.id}
          role="tab"
          type="button"
          aria-selected={activeTab === tab.id}
          onClick={() => onChange(tab.id)}
          className={`${baseStyles} ${
            activeTab === tab.id ? activeStyles : inactiveStyles
          }`}
        >
          {tab.icon && <span aria-hidden="true">{tab.icon}</span>}
          {tab.label}
        </button>
      ))}
    </div>
  );
}
