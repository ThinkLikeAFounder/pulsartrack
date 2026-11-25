export function getBackendApiBaseUrl(apiUrl: string): string {
  const normalized = apiUrl.replace(/\/+$/, "");
  return normalized.endsWith("/api") ? normalized : `${normalized}/api`;
}
