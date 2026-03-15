import { NextResponse } from 'next/server';
import { auth } from '@/auth';

// In a real production system, this route would query a time-series database
// like VictoriaMetrics or Prometheus, or read aggregated logs from APISIX.
// Here we return realistic simulated metrics based on time of day.
export async function GET() {
  const session = await auth();
  
  if (!session?.user) {
    return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
  }

  const hours = new Date().getHours();
  // Generate slightly randomized realistic data based on current hour
  const getTraffic = (base: number) => Math.floor(base + (Math.random() * 1000 - 500));
  
  const latencyData = [
    { time: '00:00', ms: 42 + Math.random()*5, requests: getTraffic(2000) },
    { time: '04:00', ms: 45 + Math.random()*5, requests: getTraffic(4000) },
    { time: '08:00', ms: 48 + Math.random()*5, requests: getTraffic(8000) },
    { time: '12:00', ms: 55 + Math.random()*15, requests: getTraffic(12000) },
    { time: '16:00', ms: 50 + Math.random()*10, requests: getTraffic(9000) },
    { time: '20:00', ms: 44 + Math.random()*5, requests: getTraffic(6000) },
    { time: 'Now', ms: 40 + Math.random()*10, requests: getTraffic(3000 + (hours * 100)) },
  ];

  const methodData = [
    { name: 'invokefunction', value: 125000 + Math.random() * 5000 },
    { name: 'getapplicationlog', value: 85000 + Math.random() * 3000 },
    { name: 'getnep17balances', value: 45000 + Math.random() * 2000 },
    { name: 'getblock', value: 25000 + Math.random() * 1000 },
    { name: 'getversion', value: 5000 + Math.random() * 500 },
  ];

  const totalReq = methodData.reduce((acc, curr) => acc + curr.value, 0);

  return NextResponse.json({
    stats: {
      totalRequests: (totalReq / 1000000).toFixed(2) + 'M',
      successRate: '99.8%',
      avgLatency: Math.floor(40 + Math.random() * 10),
      bandwidth: (42.5 + Math.random() * 2).toFixed(1)
    },
    latencyData,
    methodData,
    errorLog: [
      { time: '2 mins ago', method: 'invokefunction', error: '-500: Invalid parameters', ip: '45.22.11.X' },
      { time: '15 mins ago', method: 'getapplicationlog', error: '-100: Unknown transaction', ip: '192.168.1.X' },
      { time: '1 hour ago', method: 'sendrawtransaction', error: '-501: Insufficient GAS', ip: '8.8.4.X' },
    ]
  });
}
