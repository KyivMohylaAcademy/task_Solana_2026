import * as fs from "fs";
import * as path from "path";

const accountsFilePath = path.join(__dirname, "../accounts.json");

/**
 * Save multiple accounts to accounts.json (additive — never overwrites existing keys unless explicitly included).
 */
export function saveAccounts(next: Record<string, string>): void {
  const prev: Record<string, string> = fs.existsSync(accountsFilePath)
    ? JSON.parse(fs.readFileSync(accountsFilePath, "utf-8"))
    : {};
  fs.writeFileSync(
    accountsFilePath,
    JSON.stringify({ ...prev, ...next }, null, 2)
  );
}

/**
 * Load all accounts from accounts.json.
 */
export function loadAccounts(): Record<string, string> {
  return fs.existsSync(accountsFilePath)
    ? JSON.parse(fs.readFileSync(accountsFilePath, "utf-8"))
    : {};
}
