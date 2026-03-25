import { createServer } from "node:net";

export async function isPortAvailable(port: number, host = "0.0.0.0"): Promise<boolean> {
  return new Promise((resolve) => {
    const server = createServer();

    server.once("error", () => {
      resolve(false);
    });

    server.once("listening", () => {
      server.close();
      resolve(true);
    });

    server.listen(port, host);
  });
}

export async function findAvailablePort(startPort: number, host = "0.0.0.0"): Promise<number> {
  let port = startPort;
  while (port < 65535) {
    if (await isPortAvailable(port, host)) {
      return port;
    }
    port++;
  }
  throw new Error(`No available ports found starting from ${startPort}`);
}
