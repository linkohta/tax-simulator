// Rust 側 (src-tauri/src/tax/model.rs) と対応する型

export type Disability = "none" | "general" | "special" | "specialCohabiting";
export type WidowStatus = "none" | "widow" | "singleMother" | "singleFather";
export type DividendMode = "noDeclare" | "comprehensive" | "separate";
export type MedicalMode = "normal" | "selfMedication";
export type SocialInsuranceMode = "auto" | "manual";

export interface SpouseInput {
  age: number;
  salaryIncome: number;
  otherIncome: number;
  disability: Disability;
}

export interface DependentInput {
  name: string;
  age: number;
  salaryIncome: number;
  otherIncome: number;
  cohabitingParent: boolean;
  disability: Disability;
}

export interface FurusatoEntry {
  name: string;
  amount: number;
}

export interface TaxInput {
  age: number;
  salary: { annualIncome: number };
  socialInsurance: {
    mode: SocialInsuranceMode;
    manualAmount: number;
    monthlySalary: number;
    bonuses: number[];
    prefecture: string;
    employmentInsurance: boolean;
    childcareSupportMonths: number;
    otherAmount: number;
  };
  family: {
    hasSpouse: boolean;
    spouse: SpouseInput;
    dependents: DependentInput[];
    selfDisability: Disability;
    widowStatus: WidowStatus;
    workingStudent: boolean;
  };
  insurance: {
    lifeNew: number;
    lifeOld: number;
    medicalCare: number;
    pensionNew: number;
    pensionOld: number;
    earthquake: number;
    oldLongTerm: number;
  };
  pensionSavings: {
    ideco: number;
    smallBusinessMutualAid: number;
    corporateDcMatching: number;
  };
  medical: {
    mode: MedicalMode;
    expenses: number;
    reimbursed: number;
    otcPurchases: number;
  };
  donations: {
    furusato: FurusatoEntry[];
    oneStop: boolean;
    otherIncomeTaxOnly: number;
    otherResidentDesignated: number;
  };
  stocks: {
    dividends: number;
    dividendMode: DividendMode;
    capitalGain: number;
    declareCapitalGain: boolean;
    lossCarryforward: number;
  };
  housingLoan: { creditAmount: number; movedIn2026OrLater: boolean };
}

export interface DeductionItem {
  label: string;
  amount: number;
}

export interface TaxResult {
  income: {
    salaryIncome: number;
    salaryDeduction: number;
    incomeAdjustmentDeduction: number;
    salaryNet: number;
    dividendComprehensive: number;
    dividendSeparate: number;
    capitalGainSeparate: number;
    totalIncome: number;
    totalIncomeForTests: number;
    lossCarriedToNextYear: number;
  };
  socialInsurance: {
    health: number;
    care: number;
    childcareSupport: number;
    pension: number;
    employment: number;
    other: number;
    total: number;
    standardMonthlyHealth: number;
    standardMonthlyPension: number;
    estimated: boolean;
  };
  incomeTax: {
    deductions: DeductionItem[];
    totalDeductions: number;
    taxableOrdinary: number;
    taxableSeparate: number;
    taxOnOrdinary: number;
    taxOnSeparate: number;
    credits: DeductionItem[];
    baseTax: number;
    reconstructionTax: number;
    total: number;
    marginalRate: number;
  };
  residentTax: {
    deductions: DeductionItem[];
    totalDeductions: number;
    taxableOrdinary: number;
    taxableSeparate: number;
    incomeLevyBeforeCredits: number;
    credits: DeductionItem[];
    incomeLevy: number;
    perCapita: number;
    total: number;
    incomeLevyExempt: boolean;
    perCapitaExempt: boolean;
  };
  stockWithholding: { incomeTax: number; residentTax: number };
  furusato: {
    currentTotal: number;
    limit: number;
    currentBurden: number;
    incomeTaxReduction: number;
    residentTaxReduction: number;
  };
  summary: {
    grossIncome: number;
    socialInsurance: number;
    incomeTax: number;
    residentTax: number;
    stockWithholding: number;
    totalTax: number;
    takeHome: number;
    effectiveTaxRate: number;
    filingRequired: boolean;
  };
  warnings: string[];
}

export const newDependent = (): DependentInput => ({
  name: "",
  age: 10,
  salaryIncome: 0,
  otherIncome: 0,
  cohabitingParent: false,
  disability: "none",
});

export const defaultInput = (): TaxInput => ({
  age: 35,
  salary: { annualIncome: 5_000_000 },
  socialInsurance: {
    mode: "auto",
    manualAmount: 0,
    monthlySalary: 350_000,
    bonuses: [400_000, 400_000],
    prefecture: "東京都",
    employmentInsurance: true,
    childcareSupportMonths: 8,
    otherAmount: 0,
  },
  family: {
    hasSpouse: false,
    spouse: { age: 35, salaryIncome: 0, otherIncome: 0, disability: "none" },
    dependents: [],
    selfDisability: "none",
    widowStatus: "none",
    workingStudent: false,
  },
  insurance: {
    lifeNew: 0,
    lifeOld: 0,
    medicalCare: 0,
    pensionNew: 0,
    pensionOld: 0,
    earthquake: 0,
    oldLongTerm: 0,
  },
  pensionSavings: { ideco: 0, smallBusinessMutualAid: 0, corporateDcMatching: 0 },
  medical: { mode: "normal", expenses: 0, reimbursed: 0, otcPurchases: 0 },
  donations: { furusato: [], oneStop: true, otherIncomeTaxOnly: 0, otherResidentDesignated: 0 },
  stocks: {
    dividends: 0,
    dividendMode: "noDeclare",
    capitalGain: 0,
    declareCapitalGain: false,
    lossCarryforward: 0,
  },
  housingLoan: { creditAmount: 0, movedIn2026OrLater: false },
});
