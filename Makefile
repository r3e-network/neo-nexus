.PHONY: install build dev test typecheck lint clean clean-all start

# Development
install:
	npm install

dev:
	npm run dev

start:
	npm start

# Quality
test:
	npm test

typecheck:
	npm run typecheck

lint:
	npm run lint

# Build
build:
	npm run build

# Clean build artifacts
clean:
	rm -rf dist
	rm -rf web/dist
	rm -rf *.tsbuildinfo
	rm -rf /tmp/neo-cli-local /tmp/neo-cli-sc
	rm -f /tmp/neo-nexus-*.log /tmp/neo-nexus-*.json /tmp/tok
	@echo "Build artifacts cleaned"

# Clean everything including dependencies
clean-all: clean
	rm -rf node_modules
	rm -rf web/node_modules
	@echo "Dependencies cleaned — run 'make install' to restore"
