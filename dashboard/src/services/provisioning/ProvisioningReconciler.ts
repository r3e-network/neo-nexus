import { prisma } from '../../utils/prisma';
import { kickoffProvisioningOrder } from './ProvisioningRunner';

export async function resumePendingProvisioningOrders(
  orderIds: number[],
  kickoff: (orderId: number) => Promise<boolean> = kickoffProvisioningOrder,
) {
  let resumed = 0;
  const resumedOrderIds: number[] = [];

  for (const orderId of orderIds) {
    if (await kickoff(orderId)) {
      resumed += 1;
      resumedOrderIds.push(orderId);
    }
  }

  return {
    checked: orderIds.length,
    resumed,
    resumedOrderIds,
  };
}

export async function loadPendingProvisioningOrderIds(limit = 50) {
  const orders = await prisma.provisioningOrder.findMany({
    where: {
      status: {
        notIn: ['ready', 'failed'],
      },
      OR: [
        { nextAttemptAt: null },
        { nextAttemptAt: { lte: new Date() } },
      ],
    },
    orderBy: { nextAttemptAt: 'asc' },
    take: limit,
    select: { id: true },
  });

  return orders.map((order) => order.id);
}
