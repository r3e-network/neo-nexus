import { headers } from 'next/headers';
import { NextResponse } from 'next/server';
import Stripe from 'stripe';
import { StripeService } from '@/services/billing/StripeService';
import { getErrorMessage } from '@/server/errors';

export async function POST(req: Request) {
  if (!process.env.STRIPE_SECRET_KEY) {
    return NextResponse.json({ error: 'Stripe is not configured in this environment' }, { status: 500 });
  }

  const stripe = new Stripe(process.env.STRIPE_SECRET_KEY, {
    apiVersion: '2026-02-25.clover',
  });

  const endpointSecret = process.env.STRIPE_WEBHOOK_SECRET;

  const body = await req.text();
  const signature = (await headers()).get('stripe-signature');

  if (!endpointSecret || !signature) {
    return NextResponse.json({ error: 'Missing stripe signature or endpoint secret' }, { status: 400 });
  }

  let event: Stripe.Event;

  try {
    event = stripe.webhooks.constructEvent(body, signature, endpointSecret);
  } catch (error) {
    const message = getErrorMessage(error, 'Invalid webhook signature');
    console.error(`Webhook Error: ${message}`);
    return NextResponse.json({ error: `Webhook Error: ${message}` }, { status: 400 });
  }

  try {
    await StripeService.handleWebhook(event);
    return NextResponse.json({ received: true });
  } catch (error) {
    console.error('Error handling webhook', error);
    return NextResponse.json({ error: getErrorMessage(error, 'Handler failed') }, { status: 500 });
  }
}
