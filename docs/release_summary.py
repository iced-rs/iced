import re
import sys
import requests
from typing import List, Tuple

if len(sys.argv) < 3:
    print("Usage: python release_summary.py <personal_access_token> <previous_release_branch>")
    exit(1)

TOKEN = sys.argv[1]
HEADERS = {"Authorization": f"Bearer {TOKEN}"}
PR_COMMIT_REGEX = re.compile(r"(?i)Merge pull request #(\d+).*")

def get_merged_prs_since_release(repo: str, previous_release_branch: str) -> List[Tuple[str, int, str, str]]:
    prs = []
    compare_url = f"https://api.github.com/repos/{repo}/compare/{previous_release_branch}...master?per_page=1000"
    compare_response = requests.get(compare_url, headers=HEADERS)

    if compare_response.status_code == 200:
        compare_data = compare_response.json()
        for commit in compare_data["commits"]:
            match = PR_COMMIT_REGEX.search(commit["commit"]["message"])
            if match:
                pr_number = int(match.group(1))
                pr_url = f"https://api.github.com/repos/{repo}/pulls/{pr_number}"

                print(f"Querying PR {pr_number}")
                pr_response = requests.get(pr_url, headers=HEADERS)

                if pr_response.status_code == 200:
                    pr_data = pr_response.json()
                    prs.append((pr_data["title"], pr_number, pr_data["html_url"], pr_data["user"]["login"]))
                else:
                    print(f"Error fetching PR {pr_number}: {pr_response.status_code}")
    else:
        print(f"Error comparing branches: {compare_response.status_code}")

    return prs

def print_pr_list(prs: List[Tuple[str, int, str, str]]):
    for pr in prs:
        print(f"- {pr[0]}. [#{pr[1]}]({pr[2]})")

def print_authors(prs: List[Tuple[str, int, str, str]]):
    authors = set(pr[3] for pr in prs)
    print("\nAuthors:")
    for author in sorted(authors, key=str.casefold):
        print(f"- @{author}")

if __name__ == "__main__":
    repo = "iced-rs/iced"
    previous_release_branch = sys.argv[2]
    merged_prs = get_merged_prs_since_release(repo, previous_release_branch)
    print_pr_list(merged_prs)
    print_authors(merged_prs)
