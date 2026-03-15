import Stripe from 'stripe';
import { normalizeBillingPlan } from '@/server/organization';
import { prisma } from '@/utils/prisma';

export class StripeService {
    private static get stripe() {
        if (!process.env.STRIPE_SECRET_KEY) {
            throw new Error('STRIPE_SECRET_KEY is not defined in the environment.');
        }
        return new Stripe(process.env.STRIPE_SECRET_KEY, {
            apiVersion: '2026-02-25.clover', // Latest Stripe API version
        });
    }

    /**
     * Creates a Stripe Checkout Session for a subscription upgrade.
     */
    static async createCheckoutSession(
        organizationId: string,
        plan: 'growth' | 'dedicated',
        successUrl: string,
        cancelUrl: string,
    ): Promise<{ url: string }> {
        const priceIdGrowth = process.env.STRIPE_PRICE_ID_GROWTH;
        const priceIdDedicated = process.env.STRIPE_PRICE_ID_DEDICATED;

        if (!priceIdGrowth || !priceIdDedicated) {
            throw new Error('Stripe Price IDs are not configured in the environment.');
        }

        const prices = {
            'growth': priceIdGrowth,
            'dedicated': priceIdDedicated,
        };

        const session = await this.stripe.checkout.sessions.create({
            payment_method_types: ['card'],
            mode: 'subscription',
            line_items: [
                {
                    price: prices[plan],
                    quantity: 1,
                },
            ],
            success_url: successUrl,
            cancel_url: cancelUrl,
            client_reference_id: organizationId,
            metadata: {
                plan,
            },
        });

        if (!session.url) {
            throw new Error('Stripe did not return a checkout URL.');
        }

        return { url: session.url };
    }

    /**
     * Handles Stripe Webhook events to securely update the database when payments succeed.
     */
    static async handleWebhook(event: Stripe.Event) {
        if (event.type === 'checkout.session.completed') {
            const session = event.data.object as Stripe.Checkout.Session;
            const organizationId = session.client_reference_id;
            const customerId = typeof session.customer === 'string' ? session.customer : null;
            const plan = normalizeBillingPlan(session.metadata?.plan);

            if (organizationId) {
                await prisma.organization.update({
                    where: { id: organizationId },
                    data: {
                        stripeCustomerId: customerId ?? undefined,
                        billingPlan: plan === 'developer' ? 'growth' : plan,
                    }
                });
                console.log(`[Stripe Webhook] Organization ${organizationId} upgraded successfully.`);
            }
        }
    }
}
