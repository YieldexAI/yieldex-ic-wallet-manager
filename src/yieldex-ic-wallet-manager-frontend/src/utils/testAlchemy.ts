// Test utility to verify Alchemy API integration
import { testAlchemyConnection, fetchAllTokenBalances } from '@/services/alchemyApi';

// Test addresses with known stablecoin balances
export const TEST_ADDRESSES = {
  // Binance hot wallet (usually has large USDC/USDT balances)
  binance: '0xF977814e90dA44bFA03b6295A0616a897441aceC',
  // Kraken hot wallet
  kraken: '0x267be1c1d684f78cb4f55040373b3d0f7e71be72',
  // Circle (USDC issuer) treasury
  circle: '0x55fe002aeff02f77364de339a1292923a15844b8'
};

export const testAlchemyIntegration = async () => {
  console.log('🔥 Testing Alchemy API Integration...');

  try {
    // Test 1: Connection test
    console.log('1. Testing connection...');
    const connectionOk = await testAlchemyConnection();
    console.log(`✅ Connection test: ${connectionOk ? 'PASSED' : 'FAILED'}`);

    if (!connectionOk) {
      throw new Error('Connection test failed');
    }

    // Test 2: Fetch balances for a known address
    console.log('2. Testing balance fetching with Binance wallet...');
    const balances = await fetchAllTokenBalances(TEST_ADDRESSES.binance);
    console.log(`✅ Found ${balances.length} token balances`);

    if (balances.length > 0) {
      console.log('📊 Sample balances:');
      balances.slice(0, 3).forEach(balance => {
        console.log(`  ${balance.symbol}: ${balance.balance} (${balance.network})`);
      });
    }

    // Test 3: Test with different addresses
    console.log('3. Testing with multiple addresses...');
    for (const [name, address] of Object.entries(TEST_ADDRESSES)) {
      try {
        const addressBalances = await fetchAllTokenBalances(address);
        console.log(`✅ ${name}: ${addressBalances.length} tokens found`);
      } catch (error) {
        console.log(`❌ ${name}: Error - ${error}`);
      }
    }

    console.log('🎉 Alchemy integration test completed successfully!');
    return true;

  } catch (error) {
    console.error('❌ Alchemy integration test failed:', error);
    return false;
  }
};

// Run test when this module is imported in development
if (import.meta.env.DEV) {
  console.log('🚀 Alchemy test utility loaded. Run testAlchemyIntegration() in console to test.');
}