import { expect } from "chai";

export async function expectTxFailure<T>(promise: Promise<T>, pattern: RegExp | string) {
  try {
    await promise;
    expect.fail("Expected transaction to fail");
  } catch (err: any) {
    const message = await extractErrorMessage(err);
    if (pattern instanceof RegExp) {
      expect(message).to.match(pattern);
    } else {
      expect(message).to.include(pattern);
    }
  }
}

async function extractErrorMessage(err: any): Promise<string> {
  if (!err) {
    return "";
  }
  if (typeof err === "string") {
    return err;
  }
  if (typeof err.getLogs === "function") {
    try {
      const logs = await err.getLogs();
      if (Array.isArray(logs)) {
        return logs.join("\n");
      }
      if (logs) {
        return String(logs);
      }
    } catch {
      // fall back to other error fields
    }
  }
  if (err.logs) {
    return Array.isArray(err.logs) ? err.logs.join("\n") : String(err.logs);
  }
  if (err.error && typeof err.error.errorMessage === "string") {
    return err.error.errorMessage;
  }
  if (err.message) {
    return err.message;
  }
  return JSON.stringify(err, null, 2);
}
