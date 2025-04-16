// terms/web/step-terms.js
const next_route = "/wifi/web";

export async function activate(container) {
  const termsDiv = container.querySelector("#termsContent");
  const acceptBtn = container.querySelector("#acceptBtn");
  const declineBtn = container.querySelector("#declineBtn");

  try {
    const res = await fetch("/terms/api/userterms");
    if (!res.ok) throw new Error("Failed to load terms.");
    const text = await res.text();
    termsDiv.textContent = text;
  } catch (err) {
    console.error("Error loading terms:", err);
    termsDiv.textContent = "Error loading terms.";
    acceptBtn.disabled = true;
  }

  acceptBtn.addEventListener("click", async () => {
    try {
      const res = await fetch("/terms/api/userterms", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ accepted: true }),
      });
      if (!res.ok) throw new Error("Failed to accept terms.");

      history.pushState({}, "", next_route);
      window.dispatchEvent(new PopStateEvent("popstate"));
    } catch (err) {
      console.error("Error accepting terms:", err);
      alert("An error occurred while accepting terms.");
    }
  });

  declineBtn.addEventListener("click", () => {
    alert("You must accept the terms to proceed.");
  });
}
