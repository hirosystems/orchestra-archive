import { StyledOcticon } from '@primer/react'
import { MouseEvent } from 'react';
import styled from "styled-components";
import { useRootSelector, useRootDispatch } from "../../hooks/useRootSelector";
import { toggleBookmark, toggleNotification, isBookmarkEnabled, isNotificationEnabled } from "../../states/StateExplorerState";
import { BookmarkIcon, BookmarkFillIcon, BellIcon, BellFillIcon } from '@primer/octicons-react'

const Container = styled.div`
    color: rgb(55, 53, 47);
    font-size: 40px;
    font-weight: 700;
    letter-space: 0.03em;
    margin-top: 0px;
    cursor: default;
`

const ActionContainer = styled.div`
    display: inline;
    margin-right: 16px;
`


const Controls = (props: { fieldIdentifier: string }) => {

    const bookmarked = useRootSelector(isBookmarkEnabled);
    const notify = useRootSelector(isNotificationEnabled);
    let dispatch = useRootDispatch();

    function handleToggleBookmark(event: MouseEvent) {
        event.preventDefault();
        dispatch(toggleBookmark(props.fieldIdentifier));
    }

    function handleToggleNotify(event: MouseEvent) {
        event.preventDefault();
        dispatch(toggleNotification(props.fieldIdentifier));
    }

    let bookmarkIcon = bookmarked ? BookmarkFillIcon : BookmarkIcon;
    let notifyIcon = notify ? BellFillIcon : BellIcon;

    return (
        <Container>
            <ActionContainer onClick={handleToggleBookmark}>
                <StyledOcticon icon={bookmarkIcon} size={32} sx={{ color: 'fg.onEmphasis' }} />
            </ActionContainer>
            <ActionContainer onClick={handleToggleNotify}>
                <StyledOcticon icon={notifyIcon} size={32} sx={{ color: 'fg.onEmphasis' }} />
            </ActionContainer>
        </Container>
    );
};

export { Controls };