use alloy::signers::Signer;

use crate::create_icp_signer;

/// Let the backend canister sign a message using threshold ECDSA.
/// 
/// This function demonstrates how to sign arbitrary messages (not transactions)
/// using the ICP threshold signature scheme. The signed message can be used for:
/// - Authentication
/// - Message verification
/// - Off-chain proofs
/// - Protocol interactions that require signatures
#[ic_cdk::update]
pub async fn sign_message(message: String) -> Result<String, String> {
    let signer = create_icp_signer().await?;
    let signature = signer.sign_message(message.as_bytes()).await
        .map_err(|e| format!("Failed to sign message: {}", e))?;
    
    Ok(format!("Message: '{}' | Signature: {:?}", message, signature))
}

/// Sign a message and return both the signature and the signer's address
#[ic_cdk::update]
pub async fn sign_message_with_address(message: String) -> Result<String, String> {
    let signer = create_icp_signer().await?;
    let address = signer.address();
    let signature = signer.sign_message(message.as_bytes()).await
        .map_err(|e| format!("Failed to sign message: {}", e))?;
    
    Ok(format!(
        "Message: '{}' | Signer: {} | Signature: {:?}", 
        message, 
        address, 
        signature
    ))
}

/// Sign a hash directly (32 bytes)
#[ic_cdk::update]
pub async fn sign_hash(hash_hex: String) -> Result<String, String> {
    // Parse hex string to bytes
    let hash_bytes = hex::decode(hash_hex.trim_start_matches("0x"))
        .map_err(|e| format!("Invalid hex hash: {}", e))?;
    
    if hash_bytes.len() != 32 {
        return Err("Hash must be exactly 32 bytes".to_string());
    }
    
    let signer = create_icp_signer().await?;
    let signature = signer.sign_message(&hash_bytes).await
        .map_err(|e| format!("Failed to sign hash: {}", e))?;
    
    Ok(format!("Hash: 0x{} | Signature: {:?}", hex::encode(hash_bytes), signature))
} 