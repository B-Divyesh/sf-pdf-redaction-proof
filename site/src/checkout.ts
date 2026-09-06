export const CHECKOUT_URL = "https://api.sociobot.in/api/v1/products/pdf-redaction-proof/checkout";

/**
 * The billing service redirects a registered offer to hosted checkout. A
 * manual redirect is intentionally treated as ready: following it with fetch
 * would cross into the merchant's checkout origin and lose the CORS result.
 */
export async function checkoutIsAvailable(fetcher: typeof fetch = fetch): Promise<boolean> {
  const response = await fetcher(CHECKOUT_URL, {
    credentials: "omit",
    redirect: "manual",
  });
  return response.ok || response.type === "opaqueredirect";
}
