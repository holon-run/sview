#import <Foundation/Foundation.h>

@interface Client : NSObject
@property (nonatomic, copy) NSString *title;
- (instancetype)initWithTitle:(NSString *)title;
+ (NSString *)kind;
@end

@implementation Client
- (instancetype)initWithTitle:(NSString *)title {
    return self;
}

+ (NSString *)kind {
    return @"client";
}
@end

@protocol Screen
- (void)render;
@end
