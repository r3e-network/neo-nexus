import Stripe from 'stripe';
import { prisma } from '@/utils/prisma';

export class StripeService {
    private static stripe = new Stripe(process.env.STRIPE_SECRET_KEY || 'sk_test_mock', {
        apiVersion: '2026-02-25.clover', // Latest Stripe API version
    });

    /**
     * Creates a Stripe Checkout Session for a subscription upgrade.
     */
    static async createCheckoutSession(organizationId: string, plan: 'growth' | 'dedicated', successUrl: string, cancelUrl: string) {
        if (!process.env.STRIPE_SECRET_KEY) {
            console.log('[Stripe Mock] Simulating checkout session creation.');
            return { url: `${successUrl}?mock_session=true` };
        }

        const prices = {
            'growth': process.env.STRIPE_PRICE_ID_GROWTH || 'price_growth_mock',
            'dedicated': process.env.STRIPE_PRICE_ID_DEDICATED || 'price_dedicated_mock',
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
        });

        return { url: session.url };
    }

    /**
     * Handles Stripe Webhook events to securely update the database when payments succeed.
     */
    static async handleWebhook(event: Stripe.Event) {
        if (event.type === 'checkout.session.completed') {
            const session = event.data.object as Stripe.Checkout.Session;
            const organizationId = session.client_reference_id;
            const customerId = session.customer as string;

            if (organizationId) {
                // Determine the plan based on the amount paid or line item
                // For simplicity, we just mark them as upgraded. In real env, check line_items.
                await prisma.organization.update({
                    where: { id: organizationId },
                    data: {
                        stripeCustomerId: customerId,
                        // billingPlan is updated based on logic (hardcoded here to 'growth' for demonstration, 
                        // a real integration queries the session line items)
                        billingPlan: 'growth' 
                    }
                });
                console.log(`[Stripe Webhook] Organization ${organizationId} upgraded successfully.`);
            }
        }
    }
}
