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

    // Default to a mock organization for demo if none exists
    let orgId = (session.user as any).organizationId;
    if (!orgId) {
        const orgs = await prisma.organization.findMany({ take: 1 });
        if (orgs.length > 0) orgId = orgs[0].id;
        else throw new Error("No organization found");
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
