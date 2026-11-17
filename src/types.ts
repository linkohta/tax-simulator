// Rust 側 (src-tauri/src/tax/model.rs) と対応する型

export type Disability = "none" | "general" | "special" | "specialCohabiting";
export type WidowStatus = "none" | "widow" | "singleMother" | "singleFather";
export type DividendMode = "noDeclare" | "comprehensive" | "separate";
export type MedicalMode = "normal" | "selfMedication";
export type SocialInsuranceMode = "auto" | "manual";
export type HealthPlan = "kyokai" | "custom";

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

export type StockAccountKind = "withholding" | "noWithholding" | "general";

export interface StockAccount {
  name: string;
  kind: StockAccountKind;
  capitalGain: number;
  declare: boolean;
  /** 源泉徴収ありの特定口座に受け入れた配当等（配当等の合計の内数） */
  dividendsReceived: number;
}

/** 旧形式の capitalGain / declareCapitalGain は Rust 側が読込時に accounts へ移すので、ここには持たない */
export interface StockInput {
  dividends: number;
  dividendsFund: number;
  dividendsForeignFund: number;
  dividendsNoCredit: number;
  dividendMode: DividendMode;
  accounts: StockAccount[];
  lossCarryforward: number;
  foreignIncome: number;
  foreignTaxPaid: number;
}

export interface StockOption {
  dividendMode: DividendMode;
  declareWithholdingAccounts: boolean | null;
  incomeTax: number;
  residentTax: number;
  stockWithholding: number;
  totalTax: number;
  lossCarriedToNextYear: number;
  diffFromCurrent: number;
  isCurrent: boolean;
  isBest: boolean;
}

export type HousingLoanMode = "manual" | "auto";
export type HousingType = "certified" | "zeh" | "energySaving" | "other";

export interface HousingLoanInput {
  mode: HousingLoanMode;
  yearEndBalance: number;
  moveInYear: number;
  housingType: HousingType;
  existingHouse: boolean;
  childRearing: boolean;
  creditAmount: number;
  movedIn2026OrLater: boolean;
  specificAcquisition: boolean;
}

export interface SavingItem {
  label: string;
  incomeTax: number;
  residentTax: number;
  total: number;
}

export interface MiscIncomeInput {
  pensionIncome: number;
  sideRevenue: number;
  sideExpenses: number;
}

export interface TaxInput {
  age: number;
  salary: { annualIncome: number };
  socialInsurance: {
    mode: SocialInsuranceMode;
    manualAmount: number;
    monthlySalary: number;
    bonuses: number[];
    healthPlan: HealthPlan;
    prefecture: string;
    /** 健保組合などの本人負担分の料率（1/1000 %単位。4.5% → 4500） */
    customHealthRateX1000: number;
    customCareRateX1000: number;
    customChildcareSupportRateX1000: number;
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
  stocks: StockInput;
  housingLoan: HousingLoanInput;
  miscIncome: MiscIncomeInput;
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
    pensionNet: number;
    sideNet: number;
    miscIncome: number;
    dividendComprehensive: number;
    dividendSeparate: number;
    capitalGainSeparate: number;
    totalIncome: number;
    totalIncomeBeforeCarryover: number;
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
    foreignTaxCredit: number;
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
    monthlyFirst: number;
    monthlyRest: number;
  };
  stockWithholding: { incomeTax: number; residentTax: number };
  furusato: {
    currentTotal: number;
    limit: number;
    currentBurden: number;
    incomeTaxReduction: number;
    residentTaxReduction: number;
  };
  stockOptions: StockOption[];
  savings: SavingItem[];
  summary: {
    grossIncome: number;
    socialInsurance: number;
    incomeTax: number;
    residentTax: number;
    stockWithholding: number;
    totalTax: number;
    takeHome: number;
    takeHomeMonthly: number;
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

export const newStockAccount = (): StockAccount => ({
  name: "",
  kind: "withholding",
  capitalGain: 0,
  declare: false,
  dividendsReceived: 0,
});

export const defaultInput =(): TaxInput => ({
  age: 35,
  salary: { annualIncome: 5_000_000 },
  socialInsurance: {
    mode: "auto",
    manualAmount: 0,
    monthlySalary: 350_000,
    bonuses: [400_000, 400_000],
    healthPlan: "kyokai",
    prefecture: "東京都",
    // 健保組合などを選んだときの初期値：協会けんぽ（東京都）の本人負担分と同じ（Rust 側の Default と合わせる）
    customHealthRateX1000: 4_925,
    customCareRateX1000: 810,
    customChildcareSupportRateX1000: 115,
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
    dividendsFund: 0,
    dividendsForeignFund: 0,
    dividendsNoCredit: 0,
    dividendMode: "noDeclare",
    accounts: [],
    lossCarryforward: 0,
    foreignIncome: 0,
    foreignTaxPaid: 0,
  },
  housingLoan: {
    mode: "manual",
    yearEndBalance: 0,
    moveInYear: 2026,
    housingType: "energySaving",
    existingHouse: false,
    childRearing: false,
    creditAmount: 0,
    movedIn2026OrLater: false,
    specificAcquisition: false,
  },
  miscIncome: { pensionIncome: 0, sideRevenue: 0, sideExpenses: 0 },
});
