# Texts of the privacy findings, #936: the finding messages and the policy
# document names of `privacy::findings::finding_text`. The function stays a pure
# `*_text(kind, provider, document, en)` text function: the analysis stores
# canonical English in the JSON (`en = true`), the PDF calls it with the run
# language (#406). Which message applies stays in Rust.

## Policy documents (the link findings)
privacy-finding-doc-privacy = privacy policy
privacy-finding-doc-imprint = imprint

## Finding messages
privacy-finding-tracking-request-before-consent = { $provider } was contacted on the first visit, before any consent choice was possible.
privacy-finding-non-essential-storage-before-consent = Cookies or storage entries of { $provider } were present after the first visit, before any consent choice.
privacy-finding-visitor-ip-transfer = { $provider } was contacted on the first visit; the request transfers the visitor's IP address to a third party.
privacy-finding-third-country-provider = Providers contacted on the first visit state a seat outside the EU. This is a hint about the provider, not about the actual data flow.
privacy-finding-no-consent-platform-detected = Tracking requests were observed, but no consent platform could be detected on the first visit (requests, cookies, storage, document).
privacy-finding-policy-link-missing = No link to the { $doc } was found on the page.
privacy-finding-policy-link-unusable = The link to the { $doc } has no usable target (empty, '#' or script).
