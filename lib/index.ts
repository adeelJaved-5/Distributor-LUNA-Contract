import type { Env } from "@terra-money/terrain";
import { DistributeClient } from './clients/DistributeClient';

export class Lib extends DistributeClient {
  env: Env;

  constructor(env: Env) {
    super(env.client, env.defaultWallet, env.refs['distribute'].contractAddresses.default);
    this.env = env;
  }
};

export default Lib;
