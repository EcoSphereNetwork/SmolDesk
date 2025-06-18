export interface IConnectionAPI {
  getStatus(): Promise<string>;
  restart(): Promise<void>;
}
