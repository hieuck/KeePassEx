/**
 * KeePassEx iOS AutoFill Extension
 * Provides credentials to iOS AutoFill framework (ASCredentialProviderViewController)
 */
import AuthenticationServices
import UIKit

class CredentialProviderViewController: ASCredentialProviderViewController {

  // MARK: - Properties

  private var entries: [AutofillEntry] = []
  private var filteredEntries: [AutofillEntry] = []
  private var tableView: UITableView!
  private var searchController: UISearchController!

  // MARK: - Lifecycle

  override func viewDidLoad() {
    super.viewDidLoad()
    setupUI()
    loadEntries()
  }

  // MARK: - ASCredentialProviderViewController

  /// Called when user selects this extension for a specific service
  override func prepareCredentialList(for serviceIdentifiers: [ASCredentialServiceIdentifier]) {
    let urls = serviceIdentifiers.compactMap { $0.identifier }
    filterEntriesForUrls(urls)
  }

  /// Called for quick credential selection (no UI)
  override func provideCredentialWithoutUserInteraction(
    for credentialIdentity: ASPasswordCredentialIdentity
  ) {
    // Try to provide credential from stored identity
    let recordId = credentialIdentity.recordIdentifier ?? ""
    if let entry = entries.first(where: { $0.uuid == recordId }) {
      let credential = ASPasswordCredential(
        user: entry.username,
        password: entry.password
      )
      extensionContext.completeRequest(withSelectedCredential: credential)
    } else {
      extensionContext.cancelRequest(withError: NSError(
        domain: ASExtensionErrorDomain,
        code: ASExtensionError.credentialIdentityNotFound.rawValue
      ))
    }
  }

  // MARK: - UI Setup

  private func setupUI() {
    title = "KeePassEx"
    view.backgroundColor = .systemBackground

    // Navigation bar
    navigationItem.leftBarButtonItem = UIBarButtonItem(
      barButtonSystemItem: .cancel,
      target: self,
      action: #selector(cancel)
    )

    // Search controller
    searchController = UISearchController(searchResultsController: nil)
    searchController.searchResultsUpdater = self
    searchController.obscuresBackgroundDuringPresentation = false
    searchController.searchBar.placeholder = "Search entries..."
    navigationItem.searchController = searchController
    navigationItem.hidesSearchBarWhenScrolling = false

    // Table view
    tableView = UITableView(frame: view.bounds, style: .insetGrouped)
    tableView.autoresizingMask = [.flexibleWidth, .flexibleHeight]
    tableView.delegate = self
    tableView.dataSource = self
    tableView.register(CredentialCell.self, forCellReuseIdentifier: "CredentialCell")
    view.addSubview(tableView)
  }

  // MARK: - Data Loading

  private func loadEntries() {
    // Load from shared app group container (shared with main app)
    let sharedDefaults = UserDefaults(suiteName: "group.com.keepassex.app")
    if let data = sharedDefaults?.data(forKey: "autofill_entries"),
       let decoded = try? JSONDecoder().decode([AutofillEntry].self, from: data) {
      entries = decoded
      filteredEntries = decoded
      tableView.reloadData()
    }
  }

  private func filterEntriesForUrls(_ urls: [String]) {
    let domains = urls.compactMap { url -> String? in
      guard let host = URL(string: url)?.host else { return nil }
      return host.hasPrefix("www.") ? String(host.dropFirst(4)) : host
    }

    if domains.isEmpty {
      filteredEntries = entries
    } else {
      filteredEntries = entries.filter { entry in
        guard let entryHost = URL(string: entry.url)?.host else { return false }
        let cleanHost = entryHost.hasPrefix("www.") ? String(entryHost.dropFirst(4)) : entryHost
        return domains.contains { domain in
          cleanHost.hasSuffix(domain) || domain.hasSuffix(cleanHost)
        }
      }
    }

    tableView.reloadData()
  }

  // MARK: - Actions

  @objc private func cancel() {
    extensionContext.cancelRequest(withError: NSError(
      domain: ASExtensionErrorDomain,
      code: ASExtensionError.userCanceled.rawValue
    ))
  }

  private func selectEntry(_ entry: AutofillEntry) {
    let credential = ASPasswordCredential(
      user: entry.username,
      password: entry.password
    )
    extensionContext.completeRequest(withSelectedCredential: credential)
  }
}

// MARK: - UITableViewDataSource & Delegate

extension CredentialProviderViewController: UITableViewDataSource, UITableViewDelegate {

  func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
    return filteredEntries.count
  }

  func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
    let cell = tableView.dequeueReusableCell(withIdentifier: "CredentialCell", for: indexPath) as! CredentialCell
    let entry = filteredEntries[indexPath.row]
    cell.configure(with: entry)
    return cell
  }

  func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
    tableView.deselectRow(at: indexPath, animated: true)
    selectEntry(filteredEntries[indexPath.row])
  }

  func tableView(_ tableView: UITableView, heightForRowAt indexPath: IndexPath) -> CGFloat {
    return 60
  }
}

// MARK: - UISearchResultsUpdating

extension CredentialProviderViewController: UISearchResultsUpdating {
  func updateSearchResults(for searchController: UISearchController) {
    let query = searchController.searchBar.text?.lowercased() ?? ""
    if query.isEmpty {
      filteredEntries = entries
    } else {
      filteredEntries = entries.filter {
        $0.title.lowercased().contains(query) ||
        $0.username.lowercased().contains(query) ||
        $0.url.lowercased().contains(query)
      }
    }
    tableView.reloadData()
  }
}

// MARK: - Credential Cell

class CredentialCell: UITableViewCell {

  private let titleLabel = UILabel()
  private let usernameLabel = UILabel()
  private let iconLabel = UILabel()

  override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
    super.init(style: style, reuseIdentifier: reuseIdentifier)
    setupCell()
  }

  required init?(coder: NSCoder) { fatalError() }

  private func setupCell() {
    iconLabel.font = .systemFont(ofSize: 24)
    iconLabel.text = "🔑"
    iconLabel.translatesAutoresizingMaskIntoConstraints = false

    titleLabel.font = .systemFont(ofSize: 15, weight: .semibold)
    titleLabel.translatesAutoresizingMaskIntoConstraints = false

    usernameLabel.font = .systemFont(ofSize: 13)
    usernameLabel.textColor = .secondaryLabel
    usernameLabel.translatesAutoresizingMaskIntoConstraints = false

    let stack = UIStackView(arrangedSubviews: [titleLabel, usernameLabel])
    stack.axis = .vertical
    stack.spacing = 2
    stack.translatesAutoresizingMaskIntoConstraints = false

    contentView.addSubview(iconLabel)
    contentView.addSubview(stack)

    NSLayoutConstraint.activate([
      iconLabel.leadingAnchor.constraint(equalTo: contentView.leadingAnchor, constant: 16),
      iconLabel.centerYAnchor.constraint(equalTo: contentView.centerYAnchor),
      iconLabel.widthAnchor.constraint(equalToConstant: 36),

      stack.leadingAnchor.constraint(equalTo: iconLabel.trailingAnchor, constant: 12),
      stack.trailingAnchor.constraint(equalTo: contentView.trailingAnchor, constant: -16),
      stack.centerYAnchor.constraint(equalTo: contentView.centerYAnchor),
    ])
  }

  func configure(with entry: AutofillEntry) {
    titleLabel.text = entry.title
    usernameLabel.text = entry.username
  }
}

// MARK: - Models

struct AutofillEntry: Codable {
  let uuid: String
  let title: String
  let username: String
  let password: String
  let url: String
}
