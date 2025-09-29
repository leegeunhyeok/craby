import * as mod from '../napi/index.js';

declare function getBindings(): typeof mod;

export { getBindings };
