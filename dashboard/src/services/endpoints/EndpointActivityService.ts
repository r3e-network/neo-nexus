import type { Prisma } from '@prisma/client';
import { prisma } from '../../utils/prisma';

export type EndpointActivityCategory =
  | 'provisioning'
  | 'lifecycle'
  | 'plugin'
  | 'alert'
  | 'settings'
  | 'maintenance';

export type EndpointActivityStatus = 'pending' | 'success' | 'error' | 'info';

type EndpointActivityRow = {
  id: number;
  endpointId: number;
  category: string;
  action: string;
  status: string;
  message: string;
  metadata: Prisma.JsonValue | null;
  createdAt: Date;
};

export function toEndpointActivityView(activity: EndpointActivityRow) {
  return {
    ...activity,
    createdAt: activity.createdAt.toISOString(),
  };
}

export async function recordEndpointActivity(input: {
  endpointId: number;
  category: EndpointActivityCategory;
  action: string;
  status: EndpointActivityStatus;
  message: string;
  metadata?: Prisma.InputJsonValue;
}) {
  return prisma.endpointActivity.create({
    data: {
      endpointId: input.endpointId,
      category: input.category,
      action: input.action,
      status: input.status,
      message: input.message,
      metadata: input.metadata,
    },
  });
}
