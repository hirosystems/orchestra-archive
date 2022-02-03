import { Text, Timeline, StyledOcticon, Link } from '@primer/react'
import {PaperAirplaneIcon, RocketIcon, FlameIcon } from '@primer/octicons-react'
import styled from "styled-components"
import { NftBurnEvent, NftMintEvent, NftTransferEvent } from '../../states/NetworkingState';

export const Container = styled.div`
    color: rgb(55, 53, 47);
    font-size: 24px;
    font-weight: 600;
    letter-space: 0.03em;
    margin-top: 24px;
    cursor: default;
`

const NftEvent = (props: { event: NftMintEvent|NftTransferEvent|NftBurnEvent }) => {
    let icon = RocketIcon;
    let color = 'success.emphasis';
    let label = '';
    if ('NFTBurnEvent' in props.event) {
        icon = FlameIcon;
        color = 'danger.emphasis';
        label = `Token ${props.event.NFTBurnEvent.asset_identifier} was burnt by ${props.event.NFTBurnEvent.sender}`
    } else if ('NFTTransferEvent' in props.event) {
        icon = PaperAirplaneIcon;
        color = 'accent.emphasis';
        label = `Token ${props.event.NFTTransferEvent.asset_identifier} was transfered from ${props.event.NFTTransferEvent.sender} to ${props.event.NFTTransferEvent.recipient}`
    } else if ('NFTMintEvent' in props.event) {
        label = `Token ${props.event.NFTMintEvent.asset_identifier} was minted for by ${props.event.NFTMintEvent.recipient}`
    }

    return (
        <Timeline.Item>
            <Timeline.Badge sx={{bg: color}}>
                <StyledOcticon icon={icon} sx={{color: 'fg.onEmphasis'}} />
            </Timeline.Badge>
            <Timeline.Body>{label}</Timeline.Body>
        </Timeline.Item>
    );
};

export { NftEvent };