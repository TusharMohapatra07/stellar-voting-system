import { isConnected, requestAccess, signTransaction } from '@stellar/freighter-api';

export const connectWallet = async (): Promise<string> => {
  if (await isConnected()) {
    const access = await requestAccess() as unknown as { error?: string; address?: string };
    if (access.error) {
      throw new Error(access.error);
    }
    return access as unknown as string;
  }
  throw new Error('Freighter is not installed');
};

export const checkConnection = async (): Promise<boolean> => {
  return await isConnected();
};

export const signTx = async (xdr: string, network: string): Promise<string> => {
  const signed = await signTransaction(xdr, { network }) as unknown as { error?: string; address?: string };
  if (signed.error) {
    throw new Error(signed.error);
  }
  return signed as unknown as string;
};
