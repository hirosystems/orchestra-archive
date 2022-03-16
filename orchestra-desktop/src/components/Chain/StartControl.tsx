
import styled from "styled-components";
import { PlayIcon } from "@primer/octicons-react";
import { StyledOcticon } from "@primer/react";
import { useRootSelector, useRootDispatch } from "../../hooks/useRootSelector";
import { bootNetwork, buildNextRequest, selectIsManifestLoaded, selectIsNetworkBooting } from "../../states/NetworkingState";
import { MouseEvent } from 'react';
import { Spinner } from "@primer/react";

export const Container = styled.div`
    width: 150px;
    height: 48px;
    background-color: ${(props: { enabled: boolean }) => props.enabled ? "#6E57FF" : "rgba(255, 255, 255, 0.05)"};
    color: ${(props: { enabled: boolean }) => props.enabled ? "white" : "rgba(255, 255, 255, 0.25)"};
    border-radius: 24px;

`

export const Inner = styled.div`
    text-transform: uppercase;
    font-size: 12px;
    -webkit-user-select: none;      
    -moz-user-select: none;
    -ms-user-select: none;
    user-select: none;
    cursor: default;
    height: 100%;
    width: 100%;
    text-align: center;
    justify-content: center;
    align-items: center;
    display: flex;
    font-weight: 600;
`

const StartControl = () => {
    let dispatch = useRootDispatch();
    const isManifestLoaded = useRootSelector(selectIsManifestLoaded);
    const isNetworkBooting = useRootSelector(selectIsNetworkBooting);

    function handleBootNetwork(event: TouchEvent | MouseEvent) {
        event.preventDefault();
        if (isManifestLoaded) {
            dispatch(bootNetwork());
            dispatch(buildNextRequest(1));
        }
    }
    let enabled = isManifestLoaded && !isNetworkBooting;
    let icon = <StyledOcticon icon={PlayIcon} size={24} sx={{ mr: 2, color: enabled ? "white" : "rgba(255, 255, 255, 0.25)" }} />;
    let legend = "Start Devnet";
    if (isNetworkBooting) {
        icon = <Spinner size="small" sx={{ mr: 0 }} />;
        legend = ""
    }
    return (
        <Container enabled={enabled} onClick={handleBootNetwork}>
            <Inner onClick={handleBootNetwork}>
                {icon}
                {legend}
            </Inner>
        </Container>
    );
};

export { StartControl };