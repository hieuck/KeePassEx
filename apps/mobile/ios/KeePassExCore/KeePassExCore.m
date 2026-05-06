/**
 * KeePassEx iOS — React Native module registration (Objective-C bridge)
 */
#import <React/RCTBridgeModule.h>

@interface RCT_EXTERN_MODULE(KeePassExCore, NSObject)

// Vault
RCT_EXTERN_METHOD(openVault:(NSString *)path
                  password:(NSString *)password
                  keyFileData:(NSArray *)keyFileData
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(createVault:(NSString *)path
                  name:(NSString *)name
                  password:(NSString *)password
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(closeVault:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(lockVault)

// Entries
RCT_EXTERN_METHOD(getEntries:(NSString *)groupUuid
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(getEntry:(NSString *)uuid
                  includePassword:(BOOL)includePassword
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(getEntryPassword:(NSString *)uuid
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(createEntry:(NSDictionary *)args
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(updateEntry:(NSDictionary *)args
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(deleteEntry:(NSString *)uuid
                  permanent:(BOOL)permanent
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(searchEntries:(NSString *)query
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

// OTP
RCT_EXTERN_METHOD(generateTotp:(NSString *)entryUuid
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

// Generator
RCT_EXTERN_METHOD(generatePassword:(NSDictionary *)args
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

// Health
RCT_EXTERN_METHOD(auditVault:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

// Groups
RCT_EXTERN_METHOD(getGroups:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

@end
