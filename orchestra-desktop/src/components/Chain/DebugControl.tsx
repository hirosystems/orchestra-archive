
import styled from "styled-components";
import { PackageDependenciesIcon, PackageDependentsIcon } from "@primer/octicons-react";
import { StyledOcticon } from "@primer/react";
import { useRootSelector, useRootDispatch } from "../../hooks/useRootSelector";
import { buildNextRequest, selectNetworkPaused } from "../../states/NetworkingState";
import { MouseEvent } from 'react';

export const Container = styled.div`
    display: flex;
    gap: 2px
`

export const LeftButton = styled.div`
    width: 52px;
    height: 48px;
    background-color: ${(props: { color: string }) => props.color};
    color: rgba(255, 255, 255, 0.05);
    border-top-left-radius: 24px;
    border-bottom-left-radius: 24px;
    padding-top: 12px;
    padding-left: 16px;
`

export const RightButton = styled.div`
    width: 52px;
    height: 48px;
    background-color: ${(props: { color: string }) => props.color};
    color: "white";
    border-top-right-radius: 24px;
    border-bottom-right-radius: 24px;
    padding-top: 12px;
    padding-left: 16px;
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

const DebugControl = () => {
    let dispatch = useRootDispatch();
    const networkPaused = useRootSelector(selectNetworkPaused);

    function handleDiscardBlock(event: TouchEvent | MouseEvent) {
        event.preventDefault();
    }

    function handleMineBlock(event: TouchEvent | MouseEvent) {
        event.preventDefault();
    }

    let enabled = networkPaused;

    return (
        <Container>
            <LeftButton color={enabled ? "rgba(255, 255, 255)" : "rgba(255, 255, 255, 0.05)"}>
                <StyledOcticon icon={PackageDependenciesIcon} size={24} sx={{ mr: 2, color: enabled ? "danger.emphasis" : "rgba(255, 255, 255, 0.25)" }} />
            </LeftButton>
            <RightButton color={enabled ? "white" : "rgba(255, 255, 255, 0.05)"}>
                <StyledOcticon icon={PackageDependentsIcon} size={24} sx={{ mr: 2, color: enabled ? "success.emphasis" : "rgba(255, 255, 255, 0.25)" }} />
            </RightButton>
        </Container>
    );
};

export { DebugControl };