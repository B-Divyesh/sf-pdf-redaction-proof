# Demo sandbox

- URL: `https://pdf-redaction-proof.sociobot.in/?demo=1`
- Local URL: `http://127.0.0.1:4173/?demo=1`
- Desktop action: choose **Load sample project** on the first screen.
- Sample: `sample-board-minutes.pdf`, a 12-page board packet with one covered
  text finding and one author-metadata finding.
- Reset: choose **Reset demo** in the persistent demo banner.
- Exit: choose **Start for real** to discard the in-memory view.
- Storage: demo state is in memory only. It does not read or write app data or
  any `demo:` local-storage key. The landing page may separately cache public
  GitHub release metadata under `pdf-redaction-proof:release-metadata:v1`.
