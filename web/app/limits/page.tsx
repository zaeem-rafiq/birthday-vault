export const metadata = { title: "Limits · Birthday Vault" };

const LIMITS = [
  ["It earns nothing.", "The balance is held in dollars (USDC) and earns no interest. Inflation reduces what it can buy."],
  ["It depends on this contract and this network lasting.", "The money opens only if this contract and the NEAR network still run on the lock date, which can be 18 years away."],
  ["USDC's issuer can freeze an account.", "Circle, which issues USDC, can freeze any account that holds it, including a vault."],
  ["The code is unaudited.", "Nobody outside the team has reviewed the contracts. Do not put in money you cannot lose."],
  ["This demo uses play money and short timers.", "Everything on this site runs on NEAR's test network. Gifts here are play money, locks last minutes, and nothing has real value."],
  ["Gifts from other wallets are a preview here.", "The gift page shows a live price from NEAR Intents, but sends nothing. Intents has no test network."],
  ["A wrong child account cannot be corrected.", "The child account is fixed when the vault is made. If it is wrong, only the guardian can still move the vault."],
  ["The guardian can take the money on the lock date.", "Whoever made the vault still holds it on the lock date and can withdraw everything. The child gets the money only if the guardian hands the vault over, or the child claims it first."],
] as const;

export default function LimitsPage() {
  return (
    <div className="pt-6 md:pt-12 max-w-2xl">
      <h1 className="font-display text-4xl font-semibold">Limits</h1>
      <p className="mt-3 text-lg text-muted">What Birthday Vault does not do, in plain words.</p>
      <ul className="mt-8 space-y-6" data-testid="limits">
        {LIMITS.map(([title, text]) => (
          <li key={title} className="border-l-4 border-leaf pl-4">
            <h2 className="font-semibold text-lg">{title}</h2>
            <p className="mt-1 text-muted">{text}</p>
          </li>
        ))}
      </ul>
    </div>
  );
}
