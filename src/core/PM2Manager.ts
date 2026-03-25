import pm2 from "pm2";

export interface PM2ProcessInfo {
  pid?: number;
  name: string;
  status: "online" | "stopped" | "errored" | "unknown";
  uptime?: number;
  memory?: number;
  cpu?: number;
}

export class PM2Manager {
  private connected = false;

  async connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      pm2.connect((err) => {
        if (err) reject(err);
        else {
          this.connected = true;
          resolve();
        }
      });
    });
  }

  async disconnect(): Promise<void> {
    pm2.disconnect();
    this.connected = false;
  }

  async start(name: string, script: string, cwd: string, args?: string[]): Promise<void> {
    return new Promise((resolve, reject) => {
      pm2.start(
        {
          name,
          script,
          cwd,
          args,
          autorestart: true,
          max_restarts: 10,
        },
        (err) => {
          if (err) reject(err);
          else resolve();
        },
      );
    });
  }

  async stop(name: string): Promise<void> {
    return new Promise((resolve, reject) => {
      pm2.stop(name, (err) => {
        if (err) reject(err);
        else resolve();
      });
    });
  }

  async restart(name: string): Promise<void> {
    return new Promise((resolve, reject) => {
      pm2.restart(name, (err) => {
        if (err) reject(err);
        else resolve();
      });
    });
  }

  async delete(name: string): Promise<void> {
    return new Promise((resolve, reject) => {
      pm2.delete(name, (err) => {
        if (err) reject(err);
        else resolve();
      });
    });
  }

  async getProcessInfo(name: string): Promise<PM2ProcessInfo | null> {
    return new Promise((resolve, reject) => {
      pm2.describe(name, (err, list) => {
        if (err) return reject(err);
        if (!list || list.length === 0) return resolve(null);

        const proc = list[0];
        resolve({
          pid: proc.pid,
          name: proc.name || name,
          status:
            proc.pm2_env?.status === "online"
              ? "online"
              : proc.pm2_env?.status === "stopped"
                ? "stopped"
                : proc.pm2_env?.status === "errored"
                  ? "errored"
                  : "unknown",
          uptime: proc.pm2_env?.pm_uptime,
          memory: proc.monit?.memory,
          cpu: proc.monit?.cpu,
        });
      });
    });
  }

  async list(): Promise<PM2ProcessInfo[]> {
    return new Promise((resolve, reject) => {
      pm2.list((err, list) => {
        if (err) return reject(err);
        resolve(
          list.map((proc) => ({
            pid: proc.pid,
            name: proc.name || "",
            status:
              proc.pm2_env?.status === "online"
                ? "online"
                : proc.pm2_env?.status === "stopped"
                  ? "stopped"
                  : proc.pm2_env?.status === "errored"
                    ? "errored"
                    : "unknown",
            uptime: proc.pm2_env?.pm_uptime,
            memory: proc.monit?.memory,
            cpu: proc.monit?.cpu,
          })),
        );
      });
    });
  }
}
