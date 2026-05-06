export type PasswordStrengthLabel = "Too short" | "Weak" | "Fair" | "Strong" | "Excellent";

export interface PasswordStrength {
  score: number;
  label: PasswordStrengthLabel;
  acceptable: boolean;
  feedback: string[];
}

export function evaluatePasswordStrength(password: string): PasswordStrength {
  const feedback: string[] = [];
  const hasLower = /[a-z]/.test(password);
  const hasUpper = /[A-Z]/.test(password);
  const hasNumber = /\d/.test(password);
  const hasSymbol = /[^A-Za-z0-9]/.test(password);
  const hasMixedCase = hasLower && hasUpper;
  const hasNumberOrSymbol = hasNumber || hasSymbol;

  if (password.length < 8) {
    feedback.push("Use at least 8 characters.");
  }
  if (password.length < 12) {
    feedback.push("12 or more characters is better for admin access.");
  }
  if (!hasMixedCase) {
    feedback.push("Mix uppercase and lowercase letters.");
  }
  if (!hasNumberOrSymbol) {
    feedback.push("Add a number or symbol.");
  }

  let score = 0;
  if (password.length >= 8) score += 1;
  if (password.length >= 12) score += 1;
  if (hasMixedCase) score += 1;
  if (hasNumber) score += 1;
  if (hasSymbol) score += 1;

  score = Math.min(4, score);

  if (password.length === 0) {
    return {
      score: 0,
      label: "Too short",
      acceptable: false,
      feedback: ["Use at least 8 characters."],
    };
  }

  if (password.length < 8) {
    return {
      score: 0,
      label: "Too short",
      acceptable: false,
      feedback,
    };
  }

  if (score <= 1) {
    return { score, label: "Weak", acceptable: true, feedback };
  }
  if (score === 2) {
    return { score, label: "Fair", acceptable: true, feedback };
  }
  if (score === 3) {
    return { score, label: "Strong", acceptable: true, feedback };
  }

  return { score, label: "Excellent", acceptable: true, feedback: [] };
}

export function passwordStrengthTone(label: PasswordStrengthLabel): string {
  if (label === "Excellent" || label === "Strong") return "text-emerald-700";
  if (label === "Fair") return "text-blue-700";
  if (label === "Weak") return "text-amber-700";
  return "text-red-700";
}

export function passwordStrengthBar(label: PasswordStrengthLabel): string {
  if (label === "Excellent" || label === "Strong") return "bg-emerald-500";
  if (label === "Fair") return "bg-blue-500";
  if (label === "Weak") return "bg-amber-500";
  return "bg-red-500";
}
