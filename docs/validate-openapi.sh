#!/bin/bash
# Validate OpenAPI specification files
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OPENAPI_YAML="${SCRIPT_DIR}/openapi.yaml"
SWAGGER_HTML="${SCRIPT_DIR}/swagger-ui.html"

echo "🔍 Validating NeoNexus OpenAPI Specification..."
echo "================================================"

# Check if files exist
if [[ ! -f "$OPENAPI_YAML" ]]; then
    echo "❌ ERROR: openapi.yaml not found at $OPENAPI_YAML"
    exit 1
fi

if [[ ! -f "$SWAGGER_HTML" ]]; then
    echo "❌ ERROR: swagger-ui.html not found at $SWAGGER_HTML"
    exit 1
fi

echo "✅ Files exist check passed"

# Validate YAML syntax
echo ""
echo "📝 Checking YAML syntax..."
if command -v python3 &> /dev/null; then
    if python3 -c "import yaml; yaml.safe_load(open('$OPENAPI_YAML'))" 2>/dev/null; then
        echo "✅ YAML syntax is valid"
    else
        echo "❌ YAML syntax error detected"
        exit 1
    fi
elif command -v python &> /dev/null; then
    if python -c "import yaml; yaml.safe_load(open('$OPENAPI_YAML'))" 2>/dev/null; then
        echo "✅ YAML syntax is valid"
    else
        echo "❌ YAML syntax error detected"
        exit 1
    fi
else
    echo "⚠️  Python not available for YAML validation, skipping..."
fi

# Basic OpenAPI structure checks
echo ""
echo "🔧 Checking OpenAPI structure..."

# Required fields
REQUIRED_FIELDS=(
    "openapi:"
    "info:"
    "paths:"
)

for field in "${REQUIRED_FIELDS[@]}"; do
    if grep -q "$field" "$OPENAPI_YAML"; then
        echo "✅ Found required field: ${field%:}"
    else
        echo "❌ Missing required field: ${field%:}"
        exit 1
    fi
done

# Check version format
VERSION=$(grep "^openapi:" "$OPENAPI_YAML" | awk '{print $2}')
if [[ "$VERSION" =~ ^3\.[0-9]+\.[0-9]+$ ]]; then
    echo "✅ OpenAPI version format correct: $VERSION"
else
    echo "⚠️  Warning: Unusual OpenAPI version format: $VERSION"
fi

# Check paths count
PATH_COUNT=$(grep -c "^  /" "$OPENAPI_YAML" || echo "0")
echo "✅ Found $PATH_COUNT API endpoints"

# Check security schemes
if grep -q "securitySchemes:" "$OPENAPI_YAML"; then
    echo "✅ Security schemes defined"
    
    # Count auth types
    AUTH_TYPES=$(grep -E "^\s{2}-?bearerAuth|sessionAuth" "$OPENAPI_YAML" | wc -l)
    echo "   → $AUTH_TYPES authentication method(s)"
else
    echo "❌ No security schemes found"
    exit 1
fi

# Check schema definitions
SCHEMA_COUNT=$(grep -c "^    [A-Z]" "$OPENAPI_YAML" || echo "0")
echo "✅ Found $SCHEMA_COUNT schema definitions"

# Validate HTML structure
echo ""
echo "🎨 Validating Swagger UI HTML..."

if grep -q "<!DOCTYPE html>" "$SWAGGER_HTML"; then
    echo "✅ HTML5 doctype present"
else
    echo "❌ Missing HTML5 doctype"
    exit 1
fi

if grep -q "swagger-ui" "$SWAGGER_HTML"; then
    echo "✅ Swagger UI integration found"
else
    echo "❌ Swagger UI not integrated"
    exit 1
fi

if grep -q "./openapi.yaml" "$SWAGGER_HTML"; then
    echo "✅ OpenAPI spec link configured"
else
    echo "❌ Missing OpenAPI spec reference"
    exit 1
fi

# Summary statistics
echo ""
echo "📊 Documentation Statistics"
echo "==========================="
echo "OpenAPI YAML: $(wc -l < "$OPENAPI_YAML") lines"
echo "Swagger UI HTML: $(wc -l < "$SWAGGER_HTML") lines"

TOTAL_LINES=$(($(wc -l < "$OPENAPI_YAML") + $(wc -l < "$SWAGGER_HTML")))
echo "Total documentation files: $((TOTAL_LINES)) lines"

# Cross-reference check
echo ""
echo "🔗 Cross-referencing documentation..."

if grep -q "AGENT_API.md" "$OPENAPI_YAML"; then
    echo "✅ OpenAPI references agent API documentation"
else
    echo "⚠️  Consider adding cross-reference to AGENT_API.md"
fi

# Check for all major sections in AGENT_API.md
AGENT_API_MD="${SCRIPT_DIR}/AGENT_API.md"
if [[ -f "$AGENT_API_MD" ]]; then
    MD_LINES=$(wc -l < "$AGENT_API_MD")
    echo "Agent API Markdown: $MD_LINES lines"
    
    SECTIONS=(
        "Authentication"
        "Rate Limiting"
        "Error Handling"
        "Endpoints"
        "Example Workflows"
        "Migration Guide"
        "Troubleshooting"
    )
    
    for section in "${SECTIONS[@]}"; do
        if grep -qi "$section" "$AGENT_API_MD"; then
            echo "✅ Section covered: $section"
        else
            echo "⚠️  Missing section: $section"
        fi
    done
else
    echo "⚠️  AGENT_API.md not found (optional but recommended)"
fi

echo ""
echo "================================================"
echo "✅ All validations passed successfully!"
echo "================================================"
echo ""
echo "Documentation artifacts:"
echo "  📄 ${OPENAPI_YAML}"
echo "  🎨 ${SWAGGER_HTML}"
echo "  📖 ${AGENT_API_MD:-./AGENT_API.md}"
echo ""
echo "Next steps:"
echo "  1. Serve docs directory with web server"
echo "  2. Open swagger-ui.html in browser for interactive explorer"
echo "  3. Share AGENT_API.md with external developers"
echo ""
