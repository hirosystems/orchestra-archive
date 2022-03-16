
import styled from "styled-components";
import { PlayIcon } from "@primer/octicons-react";
import { StyledOcticon } from "@primer/react";
import { useRootSelector, useRootDispatch } from "../../hooks/useRootSelector";
import { selectNetworkBooted, bootNetwork, buildNextRequest } from "../../states/NetworkingState";
import { MouseEvent } from 'react';

export const Container = styled.div`
    width: 150px;
    height: 48px;
    background-color: ${(props: { enabled: boolean }) => props.enabled ? "#6E57FF" : "rgba(255, 255, 255, 0.1)"};
    color: ${(props: { enabled: boolean }) => props.enabled ? "white" : "rgba(110, 87, 255, 0.5)"};
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
    const networkBooted = useRootSelector(selectNetworkBooted);

    function handleBootNetwork(event: TouchEvent | MouseEvent) {
        event.preventDefault();
        dispatch(bootNetwork());
        dispatch(buildNextRequest(1));
    }
    let enabled = !networkBooted;

    return (
        <Container enabled={enabled} onClick={handleBootNetwork}>
            <Inner onClick={handleBootNetwork}>
                <StyledOcticon icon={PlayIcon} size={24} sx={{ mr: 2, color: enabled ? "white" : "rgba(110, 87, 255, 0.5)" }} />
                Booting Devnet
            </Inner>
        </Container>
    );
};

export { StartControl };