FROM mcr.microsoft.com/dotnet/runtime:8.0 AS base
WORKDIR /app
RUN apt-get update && apt-get install -y wget unzip libleveldb-dev sqlite3 libsqlite3-dev libc6-dev
RUN wget https://github.com/neo-project/neo-node/releases/download/v3.9.2/neo-cli-linux-x64.zip -O neo-cli.zip && \
    unzip neo-cli.zip -d /tmp && \
    mv /tmp/neo-cli/* /app/ && \
    rm -rf /tmp/neo-cli neo-cli.zip
COPY docker-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh
EXPOSE 10332 10333
ENTRYPOINT ["docker-entrypoint.sh"]
