ExamUQ Client macOS helper

If macOS says the app is damaged or cannot be opened, do this:

1. Drag "ExamUQ Client.app" into Applications.
2. Open Terminal.
3. Run this command:

   xattr -dr com.apple.quarantine "/Applications/ExamUQ Client.app"

4. Open the app again.

Optional:
- You can also double-click FIX_MACOS.command after moving the app to Applications.
- If you keep the app outside Applications, edit the path in the command accordingly.

Notes:
- This helper exists because the app is not signed/notarized with Apple Developer.
- The safest long-term fix is Apple notarization, but this helper is the current workaround.
