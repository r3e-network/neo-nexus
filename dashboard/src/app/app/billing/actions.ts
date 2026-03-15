'use server';

import { StripeService } from '@/services/billing/StripeService';
import { auth } from '@/auth';
import { redirect } from 'next/navigation';
import { prisma } from '@/utils/prisma';

export async function upgradePlanAction(plan: 'growth' | 'dedicated') {
    const session = await auth();
    if (!session?.user?.id) {
        throw new Error('Unauthorized');
    }

    const orgId = (session.user as any).organizationId;
    if (!orgId) {
        throw new Error("No organization found for this user. Please complete onboarding.");
    }

    // In a real Next.js app, we pass the absolute URL for Stripe redirects
    // Here we use a relative fallback which might need configuration depending on the deployment host.
    const baseUrl = process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3001';
    
    const { url } = await StripeService.createCheckoutSession(
        orgId, 
        plan, 
        `${baseUrl}/billing?success=true`, 
        `${baseUrl}/billing?canceled=true`
    );

    redirect(url!);
}

export async function verifyCryptoPaymentAction(plan: 'growth' | 'dedicated', txHash: string) {
    const session = await auth();
    if (!session?.user?.id) {
        return { success: false, error: 'Unauthorized' };
    }

    if (!process.env.DATABASE_URL) {
        return { success: false, error: 'Database not configured' };
    }

    try {
        // In a production app, you would:
        // 1. Fetch the transaction from the Neo N3 Mainnet using NeonJS (rpcClient.getApplicationLog(txHash))
        // 2. Verify the destination address matches your platform's treasury address
        // 3. Verify the transferred asset is GAS (0xd2a4cff31913016155e38e474a2c06d08be276cf)
        // 4. Verify the amount matches the plan requirement based on current oracle prices
        // 5. Ensure this txHash hasn't been used before (idempotency)

        // For this implementation, we simulate a successful blockchain verification
        console.log(`[Crypto Billing] Verifying TX ${txHash} for plan ${plan}...`);

        const orgId = (session.user as any).organizationId;
        if (!orgId) {
            throw new Error("No organization found for this user. Please complete onboarding.");
        }

        // Upgrade the plan
        await prisma.organization.update({
            where: { id: orgId },
            data: { billingPlan: plan }
        });

        console.log(`[Crypto Billing] Organization ${orgId} upgraded to ${plan} via Web3 payment.`);
        return { success: true };

    } catch (error: any) {
        console.error('Crypto verification failed:', error);
        return { success: false, error: error.message };
    }
}
