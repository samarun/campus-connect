export async function readJsonResponse<T>(response: Response): Promise<T> {
  const contentType = response.headers.get("content-type") ?? "";
  if (contentType.includes("application/json")) {
    return (await response.json()) as T;
  }

  const text = await response.text().catch(() => "");
  const cleaned = text.replace(/\s+/g, " ").trim();
  const fallback = response.statusText || "Unexpected server response";
  throw new Error(cleaned.startsWith("<") ? `${response.status} ${fallback}` : cleaned || `${response.status} ${fallback}`);
}

export function apiError(value: unknown, fallback: string) {
  if (value && typeof value === "object" && "error" in value && typeof value.error === "string") {
    return value.error;
  }
  if (value && typeof value === "object" && "warning" in value && typeof value.warning === "string") {
    return value.warning;
  }
  return fallback;
}

export function clientErrorMessage(value: unknown, fallback: string) {
  if (value instanceof Error) {
    if (value.message === "Failed to fetch" || value.message === "fetch failed") {
      return `${fallback} Check the network connection and refresh the page.`;
    }
    return value.message || fallback;
  }

  return fallback;
}
