const LABEL_NAME = "needs info"
const RE_VERSION = /Yazi\s+Version\s*:\s\d+\.\d+\.\d+\s\(/gm
const RE_DEPENDENCIES = /Dependencies\s+[/a-z]+\s*:\s/gm
const RE_CHECKLIST = /#{3}\s+Checklist\s+(?:^-\s+\[x]\s+.+?(?:\n|\r\n|$)){2}/gm

function bugReportBody(creator, content, hash) {
	if (RE_CHECKLIST.test(content) && new RegExp(` \\(${hash}[a-z]? `).test(content)) {
		return null
	}

	return `Hey @${creator}, I noticed that you did not correctly follow the issue template. Please ensure that:

- The bug can still be reproduced on the [newest nightly build](https://yazi-rs.github.io/docs/installation/#binaries).
- The debug information (\`yazi --debug\`) is updated for the newest nightly.
- The non-optional items in the checklist are checked.

Issues with \`${LABEL_NAME}\` will be marked ready once edited with the proper content, or closed after 2 days of inactivity.
`
}

function featureRequestBody(creator, content) {
	if (RE_VERSION.test(content) && RE_DEPENDENCIES.test(content) && RE_CHECKLIST.test(content)) {
		return null
	}

	return `Hey @${creator}, I noticed that you did not correctly follow the issue template. Please ensure that:

- The requested feature does not exist in the [newest nightly build](https://yazi-rs.github.io/docs/installation/#binaries).
- The debug information (\`yazi --debug\`) is updated for the newest nightly.
- The non-optional items in the checklist are checked.

Issues with \`${LABEL_NAME}\` will be marked ready once edited with the proper content, or closed after 2 days of inactivity.
`
}

module.exports = async ({ github, context, core }) => {
	async function nightlyHash() {
		try {
			const { data: tagRef } = await github.rest.git.getRef({ owner: "sxyazi", repo: "yazi", ref: "tags/nightly" })
			return tagRef.object.sha.slice(0, 7)
		} catch (e) {
			if (e.status === 404) {
				core.error("Nightly tag not found")
			} else {
				core.error(`Error fetching nightly version: ${e.message}`)
			}
			return null
		}
	}

	async function hasLabel(id, label) {
		try {
			const { data: labels } = await github.rest.issues.listLabelsOnIssue({
				...context.repo,
				issue_number: id,
			})
			return labels.some(l => l.name === label)
		} catch (e) {
			core.error(`Error checking labels: ${e.message}`)
			return false
		}
	}

	async function updateLabels(id, mark, body) {
		try {
			const marked = await hasLabel(id, LABEL_NAME)
			if (mark && !marked) {
				await github.rest.issues.addLabels({
					...context.repo,
					issue_number: id,
					labels      : [LABEL_NAME],
				})
				await github.rest.issues.createComment({
					...context.repo,
					issue_number: id,
					body,
				})
			} else if (!mark && marked) {
				await github.rest.issues.removeLabel({
					...context.repo,
					issue_number: id,
					name        : LABEL_NAME,
				})
			}
		} catch (e) {
			core.error(`Error updating labels: ${e.message}`)
		}
	}

	async function closeOldIssues() {
		try {
			const { data: issues } = await github.rest.issues.listForRepo({
				...context.repo,
				state : "open",
				labels: LABEL_NAME,
			})

			const now = new Date()
			const twoDaysAgo = new Date(now - 2 * 24 * 60 * 60 * 1000)

			for (const issue of issues) {
				const markedAt = new Date(issue.labels_at || issue.created_at)
				if (markedAt < twoDaysAgo) {
					await github.rest.issues.update({
						...context.repo,
						issue_number: issue.number,
						state       : "closed",
						state_reason: "not_planned",
					})
					await github.rest.issues.createComment({
						...context.repo,
						issue_number: issue.number,
						body        : `This issue has been automatically closed because it was marked as \`${LABEL_NAME}\` for more than 2 days without updates.
If the problem persists, please file a new issue and complete the issue template so we can capture all the details necessary to investigate further.`,
					})
				}
			}
		} catch (e) {
			core.error(`Error checking old issues: ${e.message}`)
		}
	}

	async function main() {
		const hash = await nightlyHash()
		if (!hash) return

		if (context.eventName === "schedule") {
			await closeOldIssues()
			return
		}

		if (context.eventName === "issues") {
			const id = context.payload.issue.number
			const content = context.payload.issue.body || ""
			const creator = context.payload.issue.user.login

			if (await hasLabel(id, "bug")) {
				const body = bugReportBody(creator, content, hash)
				await updateLabels(id, !!body, body)
			} else if (await hasLabel(id, "feature")) {
				const body = featureRequestBody(creator, content)
				await updateLabels(id, !!body, body)
			}
		}
	}

	await main()
}
