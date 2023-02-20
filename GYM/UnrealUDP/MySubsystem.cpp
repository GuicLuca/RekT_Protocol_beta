// Fill out your copyright notice in the Description page of Project Settings.


#include "MySubsystem.h"

#include "STSocketToolBoxSubsystem.h"
#include "Common/TcpSocketBuilder.h"
#include "Interfaces/IPv4/IPv4Endpoint.h"

bool UMySubsystem::CreateSockets(const FString& RemoteIp, const int32 RemotePort)
{
	ListenSocket = MakeUdpListenSocket("DEFAULT_SOCKET_NAME", RemoteIp, RemotePort);
	if (!ListenSocket)
	{
		UE_LOG(LogSockets,Error,TEXT("Failed to create the Socket"));
		return false;
	}
	
	UDPReceiver = new FUdpSocketReceiver(ListenSocket, FTimespan::FromMilliseconds(ThreadWaiting), *SOCKET_Name);
	
	UE_LOG(LogSockets,Display,TEXT("Socket Successfully created"));
	
	UDPReceiver->OnDataReceived().BindLambda([&](const FArrayReaderPtr& DataPtr, const FIPv4Endpoint& Endpoint)
	{
		// Process Bytes to create own TArray
		TArray<uint8> BytesReceived;
		BytesReceived.AddUninitialized(DataPtr->TotalSize());
		DataPtr->Serialize(BytesReceived.GetData(), DataPtr->TotalSize());

		// Fire own delegate
		OnRequestReceivedDelegate.Broadcast(BytesReceived,Endpoint.Address.ToString());
	});
	
	UDPReceiver->Start();
	return true;
}

bool UMySubsystem::SendDataTo(const TArray<uint8>& Bytes, const FString& RemoteIp, const int32 RemotePort)
{
	int32 BytesSent;
	auto Address = SocketSubsystem->CreateInternetAddr();
	bool bIsOk;
	Address->SetIp(*RemoteIp,bIsOk);
	Address->SetPort(RemotePort);
	if(!bIsOk) return false;
	
	const bool Result = ListenSocket->SendTo(Bytes.GetData(),Bytes.Num(),BytesSent,*Address);
	if(Result)
	{
		UE_LOG(LogSockets,Display, TEXT("Successfully sended %d Bytes"),BytesSent);
	}
	else
	{
		UE_LOG(LogSockets,Display, TEXT("There is an error when sending bytes"));
	}
	return Result;

}	


void UMySubsystem::Initialize(FSubsystemCollectionBase& Collection)
{
	Super::Initialize(Collection);
	SocketSubsystem = ISocketSubsystem::Get(PLATFORM_SOCKETSUBSYSTEM);
}

void UMySubsystem::Deinitialize()
{
	Super::Deinitialize();

	//TODO
	delete UDPReceiver;
	ListenSocket->Close();
}

FSocket* UMySubsystem::MakeUdpListenSocket(const FString &SocketName, const FString &TheIP, uint16 ThePort, uint32 BufferSize)
{
	FIPv4Address Ip;
	FIPv4Address::Parse(TheIP,Ip);
	
	RemoteAddress = FIPv4Endpoint(Ip,ThePort);
	FSocket* ListenerSocket = FUdpSocketBuilder(*SocketName)
		.AsReusable()
		.BoundToEndpoint(RemoteAddress);

	int32 NewSize = 0;
	ListenerSocket->SetReceiveBufferSize(BufferSize, NewSize);

	//Done!
	return ListenerSocket;
}
