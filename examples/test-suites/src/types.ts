export interface TestSuite {
  label: string;
  description?: string;
  action: () => Promise<any> | any;
}
