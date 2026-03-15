import { headers } from 'next/headers';
import { NextResponse } from 'next/server';
import Stripe from 'stripe';
import { StripeService } from '@/services/billing/StripeService';

const stripe = new Stripe(process.env.STRIPE_SECRET_KEY || 'sk_test_mock', {
  apiVersion: '2026-02-25.clover',
});

const endpointSecret = process.env.STRIPE_WEBHOOK_SECRET;

export async function POST(req: Request) {
  const body = await req.text();
  const signature = (await headers()).get('stripe-signature');

  if (!endpointSecret || !signature) {
    return NextResponse.json({ error: 'Missing stripe signature or endpoint secret' }, { status: 400 });
  }

  let event: Stripe.Event;

  try {
    event = stripe.webhooks.constructEvent(body, signature, endpointSecret);
  } catch (err: any) {
    console.error(`Webhook Error: ${err.message}`);
    return NextResponse.json({ error: `Webhook Error: ${err.message}` }, { status: 400 });
  }

  try {
    await StripeService.handleWebhook(event);
    return NextResponse.json({ received: true });
  } catch (error: any) {
    console.error('Error handling webhook', error);
    return NextResponse.json({ error: 'Handler failed' }, { status: 500 });
  }
}
