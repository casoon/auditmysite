# AuditMySite v2.0.0-alpha.2

Use these commands to create the GitHub Release with the correct notes via GitHub CLI (requires GH_TOKEN or an authenticated gh session):

```bash
# Ensure you are authenticated
export GH_TOKEN=<your_github_token>

# Create the release pointing to the existing tag
gh release create v2.0.0-alpha.2 \
  --title "AuditMySite v2.0.0-alpha.2" \
  --notes-file docs/release/RELEASE-NOTES-2.0.0-alpha.2.md
```

Alternatively, via API (replace placeholders):

```bash
OWNER=casoon
REPO=AuditMySite
TAG=v2.0.0-alpha.2
API=https://api.github.com/repos/$OWNER/$REPO/releases

curl -s -H "Authorization: token $GH_TOKEN" \
  -H "Accept: application/vnd.github+json" \
  -d @- $API <<'JSON'
{
  "tag_name": "v2.0.0-alpha.2",
  "name": "AuditMySite v2.0.0-alpha.2",
  "body": "$(sed 's/"/\\"/g' docs/release/RELEASE-NOTES-2.0.0-alpha.2.md)",
  "draft": false,
  "prerelease": true
}
JSON
```
