# Environment Variables Setup

This Tauri application requires Backblaze B2 cloud storage credentials to be configured via environment variables.

## Required Environment Variables

Create a `.env` file in the `src-tauri` directory with the following variables:

```env
# Required: Your B2 bucket name
B2_BUCKET=your-bucket-name

# Required: Your B2 Application Key ID
B2_KEY_ID=your-key-id

# Required: Your B2 Application Key (secret)
B2_APPLICATION_KEY=your-application-key

# Optional: B2 region (defaults to "us-west-000" if not set)
B2_REGION=us-west-000

# Optional: Custom endpoint URL (defaults to https://s3.{B2_REGION}.backblazeb2.com if not set)
# B2_ENDPOINT=https://s3.us-west-000.backblazeb2.com

# Optional: Prefix for uploaded files (defaults to folder name if not set)
# B2_PREFIX=vintage-story-saves
```

## How to Get Your B2 Credentials

1. Log in to your Backblaze account
2. Go to **App Keys** section
3. Create a new Application Key with:
   - **Read and Write** permissions
   - Access to your bucket
4. Copy the **Key ID** and **Application Key**

## Setting Environment Variables

### Option 1: Create `.env` file (Recommended for development)

Create a `.env` file in `src-tauri/` directory. Note: This file is gitignored for security.

### Option 2: Set system environment variables

You can also set these as system environment variables:

**macOS/Linux:**
```bash
export B2_BUCKET=your-bucket-name
export B2_KEY_ID=your-key-id
export B2_APPLICATION_KEY=your-application-key
export B2_REGION=us-west-000
```

**Windows (PowerShell):**
```powershell
$env:B2_BUCKET="your-bucket-name"
$env:B2_KEY_ID="your-key-id"
$env:B2_APPLICATION_KEY="your-application-key"
$env:B2_REGION="us-west-000"
```

### Option 3: Compile-time variables (for production builds)

For production builds, you can set these at compile time using `build.rs` or by setting them before running `cargo build`.

## B2 Region Values

Common Backblaze B2 regions:
- `us-west-000` (default)
- `us-west-001`
- `us-west-002`
- `us-west-003`
- `us-west-004`

Check your bucket's region in the Backblaze B2 console.

## Security Note

⚠️ **Never commit your `.env` file to version control!** The `.env` file is already in `.gitignore`.

