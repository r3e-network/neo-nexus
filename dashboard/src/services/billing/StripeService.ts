import Stripe from 'stripe';
import { normalizeBillingPlan } from '@/server/organization';
import { prisma } from '@/utils/prisma';

const STRIPE_ACTIVE_SUBSCRIPTION_STATUSES = new Set([
    'active',
    'trialing',
    'past_due',
    'unpaid',
]);

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

    static async createBillingPortalSession(
        customerId: string,
        returnUrl: string,
    ): Promise<{ url: string }> {
        const session = await this.stripe.billingPortal.sessions.create({
            customer: customerId,
            return_url: returnUrl,
        });

        if (!session.url) {
            throw new Error('Stripe did not return a billing portal URL.');
        }

        return { url: session.url };
    }

    static async listRecentInvoices(customerId: string) {
        const invoices = await this.stripe.invoices.list({
            customer: customerId,
            limit: 10,
        });

        return invoices.data.map((invoice) => ({
            id: invoice.id,
            amountPaidCents: invoice.amount_paid ?? 0,
            currency: invoice.currency ?? 'usd',
            status: invoice.status ?? 'unknown',
            hostedInvoiceUrl: invoice.hosted_invoice_url ?? null,
            createdAt: new Date(invoice.created * 1000).toISOString(),
            description: invoice.lines.data[0]?.description ?? invoice.description ?? 'Stripe invoice',
        }));
    }

    static async getCardSummary(customerId: string) {
        const paymentMethods = await this.stripe.paymentMethods.list({
            customer: customerId,
            type: 'card',
            limit: 1,
        });

        const card = paymentMethods.data[0]?.card;
        if (!card) {
            return null;
        }

        return {
            brand: card.brand ?? 'card',
            last4: card.last4 ?? '0000',
            expMonth: card.exp_month ?? 0,
            expYear: card.exp_year ?? 0,
        };
    }

    private static resolveBillingPlanFromPriceIds(priceIds: string[]) {
        const growthPriceId = process.env.STRIPE_PRICE_ID_GROWTH;
        const dedicatedPriceId = process.env.STRIPE_PRICE_ID_DEDICATED;

        if (dedicatedPriceId && priceIds.includes(dedicatedPriceId)) {
            return 'dedicated' as const;
        }

        if (growthPriceId && priceIds.includes(growthPriceId)) {
            return 'growth' as const;
        }

        return 'developer' as const;
    }

    private static async syncOrganizationPlanFromCustomer(customerId: string) {
        const subscriptions = await this.stripe.subscriptions.list({
            customer: customerId,
            status: 'all',
            limit: 20,
        });
        const activePriceIds = subscriptions.data
            .filter((subscription) => STRIPE_ACTIVE_SUBSCRIPTION_STATUSES.has(subscription.status))
            .flatMap((subscription) => subscription.items.data.map((item) => item.price.id))
            .filter(Boolean);
        const billingPlan = this.resolveBillingPlanFromPriceIds(activePriceIds);

        await prisma.organization.updateMany({
            where: { stripeCustomerId: customerId },
            data: { billingPlan },
        });
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
            }
        }

        if (
            event.type === 'customer.subscription.created'
            || event.type === 'customer.subscription.updated'
            || event.type === 'customer.subscription.deleted'
        ) {
            const subscription = event.data.object as Stripe.Subscription;
            const customerId = typeof subscription.customer === 'string'
                ? subscription.customer
                : subscription.customer?.id;

            if (customerId) {
                await this.syncOrganizationPlanFromCustomer(customerId);
            }
        }
    }
}
