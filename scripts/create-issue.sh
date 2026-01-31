#!/bin/bash
# Copyright 2023 RobustMQ Team
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.


# Script to create GitHub issues using GitHub API
# Usage: 
#   ./scripts/create-issue.sh --title "Issue Title" --body-file issue.md --labels "bug,mqtt"
#   ./scripts/create-issue.sh -t "Issue Title" -b "Issue body text" -l "enhancement"
#   ./scripts/create-issue.sh --interactive

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
REPO="robustmq/robustmq"
TITLE=""
BODY=""
BODY_FILE=""
LABELS=""
INTERACTIVE=false

# Help function
show_help() {
    cat << EOF
${BLUE}GitHub Issue Creation Script${NC}

${YELLOW}Usage:${NC}
  $0 [OPTIONS]

${YELLOW}Options:${NC}
  -t, --title TITLE           Issue title (required unless --interactive)
  -b, --body TEXT             Issue body text
  -f, --body-file FILE        Read issue body from markdown file
  -l, --labels LABELS         Comma-separated labels (e.g., "bug,mqtt,priority:high")
  -r, --repo OWNER/REPO       Repository (default: robustmq/robustmq)
  -i, --interactive           Interactive mode - prompt for all inputs
  -h, --help                  Show this help message

${YELLOW}Environment Variables:${NC}
  GITHUB_TOKEN               GitHub Personal Access Token (required)

${YELLOW}Examples:${NC}
  # Create issue with inline body
  $0 -t "Fix memory leak" -b "Description here" -l "bug,mqtt"

  # Create issue from markdown file
  $0 -t "Feature: Multi-tenant" -f docs/multi-tenant.md -l "enhancement"

  # Interactive mode
  $0 --interactive

  # Use custom repository
  $0 -t "Issue title" -b "Body" -r "myorg/myrepo"

${YELLOW}Note:${NC}
  Get your GitHub token at: https://github.com/settings/tokens
  Required permission: repo (Full control of private repositories)
EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--title)
            TITLE="$2"
            shift 2
            ;;
        -b|--body)
            BODY="$2"
            shift 2
            ;;
        -f|--body-file)
            BODY_FILE="$2"
            shift 2
            ;;
        -l|--labels)
            LABELS="$2"
            shift 2
            ;;
        -r|--repo)
            REPO="$2"
            shift 2
            ;;
        -i|--interactive)
            INTERACTIVE=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

# Check if GITHUB_TOKEN is set
if [ -z "$GITHUB_TOKEN" ]; then
    echo -e "${RED}Error: GITHUB_TOKEN environment variable is not set${NC}"
    echo "Please set it with: export GITHUB_TOKEN=your_github_token"
    echo "Get your token at: https://github.com/settings/tokens"
    exit 1
fi

# Interactive mode
if [ "$INTERACTIVE" = true ]; then
    echo -e "${BLUE}=== Interactive Issue Creation ===${NC}"
    echo ""
    
    read -p "$(echo -e ${YELLOW}Repository [robustmq/robustmq]:${NC} )" input_repo
    if [ -n "$input_repo" ]; then
        REPO="$input_repo"
    fi
    
    read -p "$(echo -e ${YELLOW}Issue Title:${NC} )" TITLE
    
    echo -e "${YELLOW}Issue Body (press Ctrl+D when done, or provide file path):${NC}"
    read -p "Body file path (leave empty to type inline): " input_file
    
    if [ -n "$input_file" ]; then
        BODY_FILE="$input_file"
    else
        echo "Type issue body (press Ctrl+D when done):"
        BODY=$(cat)
    fi
    
    read -p "$(echo -e ${YELLOW}Labels (comma-separated):${NC} )" LABELS
    echo ""
fi

# Validate required parameters
if [ -z "$TITLE" ]; then
    echo -e "${RED}Error: Issue title is required${NC}"
    echo "Use --title or --interactive mode"
    exit 1
fi

# Read body from file if specified
if [ -n "$BODY_FILE" ]; then
    if [ ! -f "$BODY_FILE" ]; then
        echo -e "${RED}Error: Body file not found: $BODY_FILE${NC}"
        exit 1
    fi
    BODY=$(cat "$BODY_FILE")
fi

# Validate body
if [ -z "$BODY" ]; then
    echo -e "${RED}Error: Issue body is required${NC}"
    echo "Use --body, --body-file, or --interactive mode"
    exit 1
fi

# Convert comma-separated labels to JSON array
if [ -n "$LABELS" ]; then
    # Split labels by comma and create JSON array
    IFS=',' read -ra LABEL_ARRAY <<< "$LABELS"
    LABELS_JSON="["
    for i in "${!LABEL_ARRAY[@]}"; do
        # Trim whitespace
        label=$(echo "${LABEL_ARRAY[$i]}" | xargs)
        LABELS_JSON+="\"$label\""
        if [ $i -lt $((${#LABEL_ARRAY[@]} - 1)) ]; then
            LABELS_JSON+=","
        fi
    done
    LABELS_JSON+="]"
else
    LABELS_JSON="[]"
fi

# Create JSON payload
JSON_PAYLOAD=$(jq -n \
    --arg title "$TITLE" \
    --arg body "$BODY" \
    --argjson labels "$LABELS_JSON" \
    '{title: $title, body: $body, labels: $labels}')

echo -e "${YELLOW}Creating GitHub issue...${NC}"
echo -e "Repository: ${GREEN}$REPO${NC}"
echo -e "Title: ${GREEN}$TITLE${NC}"
echo ""

# Create the issue
RESPONSE=$(curl -s -w "\n%{http_code}" \
    -X POST \
    -H "Accept: application/vnd.github.v3+json" \
    -H "Authorization: token $GITHUB_TOKEN" \
    "https://api.github.com/repos/$REPO/issues" \
    -d "$JSON_PAYLOAD")

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
RESPONSE_BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" = "201" ]; then
    ISSUE_URL=$(echo "$RESPONSE_BODY" | jq -r '.html_url')
    ISSUE_NUMBER=$(echo "$RESPONSE_BODY" | jq -r '.number')
    echo -e "${GREEN}✓ Issue created successfully!${NC}"
    echo -e "Issue #${ISSUE_NUMBER}: ${ISSUE_URL}"
else
    echo -e "${RED}✗ Failed to create issue${NC}"
    echo -e "HTTP Status: $HTTP_CODE"
    echo "$RESPONSE_BODY" | jq '.'
    exit 1
fi
