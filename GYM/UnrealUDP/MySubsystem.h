// Fill out your copyright notice in the Description page of Project Settings.

#pragma once

#include "CoreMinimal.h"
#include "Common/UdpSocketReceiver.h"
#include "UObject/Object.h"
#include "MySubsystem.generated.h"

DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(FOnRequestReceived, const TArray<uint8>&, Bytes, const FString &, IpAdress);
/**
 * 
 */
UCLASS()
class SOCKETSTOOLBOX_API UMySubsystem : public UGameInstanceSubsystem
{
	GENERATED_BODY()
public:
	UFUNCTION(BlueprintCallable)
	bool CreateSockets(const FString& RemoteIp, const int32 RemotePort);
	
	UFUNCTION(BlueprintCallable)
	bool SendDataTo(const TArray<uint8>& Bytes, const FString& RemoteIp, const int32 RemotePort);
	virtual void Initialize(FSubsystemCollectionBase& Collection) override;
	virtual void Deinitialize() override;
	
private:
	FSocket* MakeUdpListenSocket(const FString &SocketName, const FString &TheIP, uint16 ThePort, uint32 BufferSize = 2*1024*1024);

public:
	

	UPROPERTY(BlueprintAssignable)
	FOnRequestReceived OnRequestReceivedDelegate;

private:
	//TODO
	FUdpSocketReceiver * UDPReceiver;

	FSocket * ListenSocket;
	FIPv4Endpoint RemoteAddress;
	ISocketSubsystem * SocketSubsystem;
	
	inline const static FString SOCKET_Name = "DEFAULT_SOCKET_NAME";
	inline constexpr static int32 ThreadWaiting = 100;
};


