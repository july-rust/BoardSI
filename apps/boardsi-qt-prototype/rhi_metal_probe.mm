// A_019 native Metal adapter identity probe. Called ONLY on the Qt scene
// graph render thread through QSGRendererInterface::DeviceResource.
// This is a public Objective-C++/Metal API, not a Qt private RHI dependency.
#import <Foundation/Foundation.h>
#import <Metal/Metal.h>

extern "C" bool boardSiMetalDeviceConfirmed(void *opaqueDevice) {
    if (!opaqueDevice) return false;
    id<MTLDevice> device = (__bridge id<MTLDevice>)opaqueDevice;
    // This verifies an actual MTLDevice owned by the scene graph backend.
    // An API selection or a frameSwapped signal alone is insufficient proof.
    return [device conformsToProtocol:@protocol(MTLDevice)] &&
           [device name] != nil && [[device name] length] > 0;
}
