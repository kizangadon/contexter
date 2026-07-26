import { agentsHandlers } from './agents';
import { analyticsHandlers } from './analytics';
import { auditHandlers } from './audit';
import { correlationHandlers } from './correlation';
import { efficiencyHandlers } from './efficiency';
import { exportsHandlers } from './exports';
import { feedbackHandlers } from './feedback';
import { memoriesHandlers } from './memories';
import { notificationsHandlers } from './notifications';
import { onboardingHandlers } from './onboarding';
import { searchHandlers } from './search';
import { sessionsHandlers } from './sessions';
import { settingsHandlers } from './settings';
import { skillsHandlers } from './skills';

export const handlers = [
  ...sessionsHandlers,
  ...memoriesHandlers,
  ...agentsHandlers,
  ...skillsHandlers,
  ...efficiencyHandlers,
  ...analyticsHandlers,
  ...settingsHandlers,
  ...notificationsHandlers,
  ...searchHandlers,
  ...feedbackHandlers,
  ...exportsHandlers,
  ...correlationHandlers,
  ...auditHandlers,
  ...onboardingHandlers,
];
