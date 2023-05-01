import { Config } from "../Types.js";

const TestConfig: Config = {
  serviceName: "test",
  dbsDir: "./dbs-test",
  cacheTtlInSeconds: 60,
  notifyLatencyInMs: 10,
  serviceDbPath: "./dbs-test/service.db",
};

export default TestConfig;
