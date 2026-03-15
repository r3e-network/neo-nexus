import { NextResponse } from 'next/server';
import { prisma } from '@/utils/prisma';
import { auth } from '@/auth';

export async function GET() {
  const session = await auth();
  
  if (!session?.user) {
    return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
  }

  // Find organization ID 
  let orgId = (session.user as any).organizationId;
  if (!orgId) {
    // If no org ID is attached, try to find one associated with the user
    const userDb = await prisma.user.findUnique({
      where: { id: session.user.id },
      include: { organization: true }
    });
    if (userDb?.organizationId) {
      orgId = userDb.organizationId;
    } else {
        // Find default or fallback for demo
        const orgs = await prisma.organization.findMany({ take: 1 });
        if (orgs.length > 0) orgId = orgs[0].id;
        else return NextResponse.json([]); // return empty if no org exists at all
    }
  }

  try {
    const endpoints = await prisma.endpoint.findMany({
      where: { organizationId: orgId },
      orderBy: { createdAt: 'desc' }
    });
    return NextResponse.json(endpoints);
  } catch (error) {
    console.error('Error fetching endpoints:', error);
    return NextResponse.json({ error: 'Internal Server Error' }, { status: 500 });
  }
}
